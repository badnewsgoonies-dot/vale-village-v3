"""
Sprite generator — Imagen 3 via Vertex AI + Gemini vision evaluation.

Generates game sprites from TOML manifests, downscales, removes background,
evaluates with VLM, saves to asset directories.

Requires same env vars as gemini_vertex.py:
    GEMINI_OAUTH_CLIENT_ID, GEMINI_OAUTH_CLIENT_SECRET,
    GEMINI_OAUTH_REFRESH_TOKEN, GEMINI_VERTEX_PROJECT
"""

import base64
import json
import os
import sys
import urllib.request
import urllib.parse
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gemini_vertex import _get_access_token, gemini_vision_json


# ── Imagen 3 generation ─────────────────────────────────────────────

def generate_sprite(prompt, output_path, aspect_ratio="1:1", model="imagen-3.0-generate-002"):
    """Generate a sprite via Imagen 3 and save to disk."""
    token = _get_access_token()
    project = os.environ["GEMINI_VERTEX_PROJECT"]

    url = (
        f"https://us-central1-aiplatform.googleapis.com/v1/projects/{project}"
        f"/locations/us-central1/publishers/google/models/{model}:predict"
    )

    body = json.dumps({
        "instances": [{"prompt": prompt}],
        "parameters": {
            "sampleCount": 1,
            "aspectRatio": aspect_ratio,
        }
    }).encode()

    req = urllib.request.Request(url, data=body, headers={
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    })

    with urllib.request.urlopen(req, timeout=60) as resp:
        data = json.loads(resp.read())

    b64 = data["predictions"][0]["bytesBase64Encoded"]
    img_bytes = base64.b64decode(b64)

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, "wb") as f:
        f.write(img_bytes)

    return output_path


# ── Post-processing ──────────────────────────────────────────────────

def remove_background(input_path, output_path, threshold=30):
    """Remove near-uniform background by flood-filling corners with transparency."""
    from PIL import Image
    import numpy as np

    img = Image.open(input_path).convert("RGBA")
    arr = np.array(img)

    # Sample corners for background color
    corners = [arr[0, 0, :3], arr[0, -1, :3], arr[-1, 0, :3], arr[-1, -1, :3]]
    bg_color = np.mean(corners, axis=0).astype(int)

    # Create mask: pixels close to bg_color become transparent
    diff = np.abs(arr[:, :, :3].astype(int) - bg_color)
    mask = np.all(diff < threshold, axis=2)
    arr[mask, 3] = 0

    result = Image.fromarray(arr)
    result.save(output_path)
    return output_path


def downscale(input_path, output_path, target_size=(64, 64), method="nearest"):
    """Downscale sprite. Use 'nearest' for pixel art, 'lanczos' for smooth."""
    from PIL import Image

    resampling = {
        "nearest": Image.Resampling.NEAREST,
        "lanczos": Image.Resampling.LANCZOS,
    }

    img = Image.open(input_path)
    resized = img.resize(target_size, resampling.get(method, Image.Resampling.NEAREST))
    resized.save(output_path)
    return output_path


# ── VLM Evaluation ───────────────────────────────────────────────────

def evaluate_sprite(image_path, entity_name, entity_type="enemy"):
    """Evaluate generated sprite quality via Gemini vision."""
    schema = {
        "type": "OBJECT",
        "properties": {
            "is_pixel_art": {"type": "BOOLEAN"},
            "matches_description": {"type": "BOOLEAN"},
            "has_transparent_background": {"type": "BOOLEAN"},
            "quality_score": {"type": "INTEGER"},
            "issues": {"type": "ARRAY", "items": {"type": "STRING"}},
            "verdict": {"type": "STRING"},
        },
        "required": ["is_pixel_art", "matches_description", "quality_score", "verdict"],
    }

    prompt = (
        f"Evaluate this image as a {entity_type} sprite named '{entity_name}' "
        f"for a tactical RPG (Golden Sun style). "
        f"Score quality 1-10. List any issues. "
        f"Verdict: PASS (>=7) or REDO (<7)."
    )

    return gemini_vision_json(image_path, prompt, schema, model="gemini-3-flash-preview")


# ── Batch generation from manifest ───────────────────────────────────

def generate_from_manifest(manifest_path, asset_root, target_size=(64, 64),
                           limit=None, skip_existing=True):
    """Generate sprites for all entries in a TOML manifest."""
    results = []

    with open(manifest_path) as f:
        content = f.read()

    entries = parse_enemy_manifest(content) if "sprite_idle" in content else parse_djinn_manifest(content)

    for i, entry in enumerate(entries):
        if limit and i >= limit:
            break

        sprite_path = entry["sprite_path"]
        full_path = os.path.join(asset_root, sprite_path.replace("assets/", ""))

        if skip_existing and os.path.exists(full_path):
            results.append({"id": entry["id"], "status": "skipped", "path": full_path})
            continue

        print(f"[{i+1}/{len(entries)}] Generating {entry['id']}...")

        # Generate
        prompt = build_prompt(entry)
        raw_path = full_path.replace(".png", "_raw.png")

        try:
            generate_sprite(prompt, raw_path)

            # Remove background
            rgba_path = full_path.replace(".png", "_rgba.png")
            remove_background(raw_path, rgba_path)

            # Downscale
            downscale(rgba_path, full_path, target_size, method="lanczos")

            # Evaluate
            eval_result = evaluate_sprite(full_path, entry["name"])

            results.append({
                "id": entry["id"],
                "status": "generated",
                "path": full_path,
                "quality": eval_result.get("quality_score", 0),
                "verdict": eval_result.get("verdict", "UNKNOWN"),
                "issues": eval_result.get("issues", []),
            })

            # Cleanup raw files
            for tmp in [raw_path, rgba_path]:
                if os.path.exists(tmp) and tmp != full_path:
                    os.remove(tmp)

            print(f"  → {eval_result.get('verdict', '?')} (quality: {eval_result.get('quality_score', '?')})")

        except Exception as e:
            results.append({"id": entry["id"], "status": "error", "error": str(e)})
            print(f"  → ERROR: {e}")

    return results


def build_prompt(entry):
    """Build an Imagen prompt from manifest entry."""
    name = entry.get("name", entry.get("id", "monster"))
    element = entry.get("element", "")
    sprite_type = entry.get("sprite_type", "idle")

    element_colors = {
        "Venus": "earth-toned, green and brown",
        "Mars": "fiery, red and orange",
        "Mercury": "icy, blue and silver",
        "Jupiter": "electric, purple and gold",
    }
    color_hint = element_colors.get(element, "")

    return (
        f"A pixel art sprite of {name} for a tactical RPG game. "
        f"{'Element: ' + element + '. Colors: ' + color_hint + '. ' if element else ''}"
        f"{'Idle pose. ' if sprite_type == 'idle' else 'Attack pose. '}"
        f"Side view facing right. Clean pixel art style with clear outlines. "
        f"Dark or black background (will be removed). "
        f"Fantasy game aesthetic similar to Golden Sun GBA games."
    )


def parse_enemy_manifest(content):
    """Parse enemy TOML manifest."""
    entries = []
    current = {}
    for line in content.split("\n"):
        line = line.strip()
        if line.startswith("[entities."):
            if current:
                entries.append(current)
            eid = line.split("[entities.")[1].rstrip("]")
            current = {"id": eid}
        elif "=" in line and current:
            key, val = line.split("=", 1)
            key = key.strip()
            val = val.strip().strip('"')
            if key == "display_name":
                current["name"] = val
            elif key == "sprite_idle":
                current["sprite_path"] = val
                current["sprite_type"] = "idle"
            elif key == "data":
                if "#" in val:
                    current["data_ref"] = val.split("#")[1]
    if current:
        entries.append(current)
    return entries


def parse_djinn_manifest(content):
    """Parse djinn TOML manifest."""
    entries = []
    current = {}
    for line in content.split("\n"):
        line = line.strip()
        if line == "[[djinn]]":
            if current:
                entries.append(current)
            current = {}
        elif "=" in line and current:
            key, val = line.split("=", 1)
            key = key.strip()
            val = val.strip().strip('"')
            current[key] = val
            if key == "sprite":
                current["sprite_path"] = val
                current["sprite_type"] = "idle"
            if key == "display_name":
                current["name"] = val
    if current:
        entries.append(current)
    return entries


# ── CLI ──────────────────────────────────────────────────────────────

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage:")
        print("  sprite_gen.py single <prompt> <output.png>")
        print("  sprite_gen.py manifest <manifest.toml> <asset_root> [--limit N]")
        print("  sprite_gen.py evaluate <image.png> <name>")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "single":
        prompt = sys.argv[2]
        output = sys.argv[3]
        generate_sprite(prompt, output)
        print(f"Generated: {output}")

    elif cmd == "manifest":
        manifest = sys.argv[2]
        asset_root = sys.argv[3]
        limit = None
        if "--limit" in sys.argv:
            limit = int(sys.argv[sys.argv.index("--limit") + 1])
        results = generate_from_manifest(manifest, asset_root, limit=limit)
        print(json.dumps(results, indent=2))

    elif cmd == "evaluate":
        image = sys.argv[2]
        name = sys.argv[3]
        result = evaluate_sprite(image, name)
        print(json.dumps(result, indent=2))
