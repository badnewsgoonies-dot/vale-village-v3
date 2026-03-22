#!/usr/bin/env python3
"""PixelLab sprite pipeline for Vale Village v3.

Generates idle, attack, and hit sprites for all enemies in the manifest.
Uses style reference from a seed sprite for visual consistency.

Usage:
    python3 tools/pixellab_pipeline.py --test          # Generate 1 test enemy
    python3 tools/pixellab_pipeline.py --batch 10      # Generate 10 enemies
    python3 tools/pixellab_pipeline.py --all            # Generate all missing
    python3 tools/pixellab_pipeline.py --list-missing   # Show what's needed
"""

import argparse
import base64
import io
import json
import os
import sys
import time
import urllib.request
import urllib.error

from PIL import Image

# ── Config ───────────────────────────────────────────────────────────

API_KEY = os.environ.get("PIXELLAB_API_KEY", "")
BASE_URL = "https://api.pixellab.ai/v2"
MANIFEST_PATH = "status/draft-enemy-manifest.toml"
SPRITE_DIR = "assets/sprites/enemies"
SPRITE_SIZE = 64  # All sprites at 64x64
STYLE_REF = "assets/sprites/enemies/bandit_idle.png"  # Style anchor

# ── API Helpers ──────────────────────────────────────────────────────

def api_post(endpoint, payload):
    """POST to PixelLab API, return parsed JSON."""
    req = urllib.request.Request(
        f"{BASE_URL}/{endpoint}",
        data=json.dumps(payload).encode(),
        headers={
            "Authorization": f"Bearer {API_KEY}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        resp = urllib.request.urlopen(req, timeout=30)
        return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        body = e.read().decode()
        print(f"  HTTP {e.code}: {body[:200]}")
        return None


def poll_job(job_id, timeout=120):
    """Poll a background job until completed or timeout."""
    for _ in range(timeout // 4):
        time.sleep(4)
        req = urllib.request.Request(
            f"{BASE_URL}/background-jobs/{job_id}",
            headers={"Authorization": f"Bearer {API_KEY}"},
        )
        try:
            resp = json.loads(urllib.request.urlopen(req, timeout=15).read())
        except Exception as e:
            print(f"  Poll error: {e}")
            continue

        if resp["status"] == "completed":
            return resp.get("last_response", resp)
        if resp["status"] == "failed":
            print(f"  Job FAILED: {json.dumps(resp, indent=2)[:300]}")
            return None
    print("  TIMEOUT waiting for job")
    return None


def save_rgba_image(img_info, path):
    """Save a PixelLab RGBA bytes image as PNG."""
    raw = base64.b64decode(img_info["base64"])
    w = img_info["width"]
    h = len(raw) // (w * 4)
    img = Image.frombytes("RGBA", (w, h), raw)
    img.save(path)
    return img


def load_png_b64(path):
    """Load a PNG file as base64 string."""
    with open(path, "rb") as f:
        return base64.b64encode(f.read()).decode()


def downscale_to_b64(path, target_size):
    """Downscale a PNG to target size and return base64."""
    img = Image.open(path).convert("RGBA")
    if img.size != (target_size, target_size):
        img = img.resize((target_size, target_size), Image.NEAREST)
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    return base64.b64encode(buf.getvalue()).decode()


# ── Manifest Parser ──────────────────────────────────────────────────

def parse_manifest():
    """Parse enemy manifest, return list of {id, name, sprite_idle, sprite_attack, sprite_hit}."""
    enemies = []
    with open(MANIFEST_PATH) as f:
        current = {}
        for line in f:
            line = line.strip()
            if line.startswith("[entities."):
                if current.get("id"):
                    enemies.append(current)
                eid = line.split("[entities.")[1].rstrip("]").strip()
                current = {"id": eid}
            elif "=" in line and current:
                key, _, val = line.partition("=")
                key = key.strip()
                val = val.strip().strip('"')
                if key == "display_name":
                    current["name"] = val
                elif key == "sprite_idle":
                    current["sprite_idle"] = val
                elif key == "sprite_attack":
                    current["sprite_attack"] = val
                elif key == "sprite_hit":
                    current["sprite_hit"] = val
        if current.get("id"):
            enemies.append(current)
    return enemies


def find_missing(enemies):
    """Return enemies missing any of idle/attack/hit."""
    missing = []
    for e in enemies:
        idle = e.get("sprite_idle", "")
        attack = e.get("sprite_attack", "")
        hit = e.get("sprite_hit", "")
        needs = []
        if not idle or not os.path.exists(idle):
            needs.append("idle")
        if not attack or not os.path.exists(attack):
            needs.append("attack")
        if not hit or not os.path.exists(hit):
            needs.append("hit")
        if needs:
            missing.append((e, needs))
    return missing


# ── Generation ───────────────────────────────────────────────────────

def generate_idle(enemy, style_b64):
    """Generate idle sprite for one enemy."""
    name = enemy.get("name", enemy["id"])
    prompt = f"{name}, fantasy RPG pixel art enemy sprite, facing east, idle stance, dark background"

    resp = api_post("generate-image-v2", {
        "description": prompt,
        "image_size": {"width": SPRITE_SIZE, "height": SPRITE_SIZE},
        "no_background": True,
        "style_image": {
            "image": {"type": "base64", "base64": style_b64, "format": "png"},
            "size": {"width": SPRITE_SIZE, "height": SPRITE_SIZE},
        },
    })
    if not resp or "background_job_id" not in resp:
        return False

    result = poll_job(resp["background_job_id"])
    if not result or "images" not in result:
        return False

    path = enemy.get("sprite_idle", os.path.join(SPRITE_DIR, f"{enemy['id'].replace('-','_')}_idle.png"))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    save_rgba_image(result["images"][0], path)
    return True


def generate_attack_hit(enemy, pose):
    """Generate attack or hit animation, save best keyframe."""
    idle_path = enemy.get("sprite_idle", os.path.join(SPRITE_DIR, f"{enemy['id'].replace('-','_')}_idle.png"))
    if not os.path.exists(idle_path):
        print(f"  No idle sprite for {enemy['id']}, skipping {pose}")
        return False

    ref_b64 = load_png_b64(idle_path)
    action = "attacking with a powerful strike, lunging forward" if pose == "attack" else "getting hit, recoiling backward in pain"

    resp = api_post("animate-with-text-v2", {
        "reference_image": {"type": "base64", "base64": ref_b64, "format": "png"},
        "reference_image_size": {"width": SPRITE_SIZE, "height": SPRITE_SIZE},
        "action": action,
        "image_size": {"width": SPRITE_SIZE, "height": SPRITE_SIZE},
        "no_background": True,
        "view": "side",
        "direction": "east",
    })
    if not resp or "background_job_id" not in resp:
        return False

    result = poll_job(resp["background_job_id"])
    if not result or "images" not in result:
        return False

    frames = result["images"]
    if pose == "attack":
        idx = len(frames) // 3
    else:
        idx = int(len(frames) * 0.4)

    key = f"sprite_{pose}"
    path = enemy.get(key, os.path.join(SPRITE_DIR, f"{enemy['id'].replace('-','_')}_{pose}.png"))
    os.makedirs(os.path.dirname(path), exist_ok=True)
    save_rgba_image(frames[idx], path)
    return True


def generate_enemy(enemy, style_b64, needs):
    """Generate all missing poses for one enemy."""
    name = enemy.get("name", enemy["id"])
    print(f"  {name} — needs: {', '.join(needs)}")

    if "idle" in needs:
        print(f"    idle...", end=" ", flush=True)
        ok = generate_idle(enemy, style_b64)
        print("OK" if ok else "FAIL")
        if not ok:
            return False
        time.sleep(2)

    if "attack" in needs:
        print(f"    attack...", end=" ", flush=True)
        ok = generate_attack_hit(enemy, "attack")
        print("OK" if ok else "FAIL")
        time.sleep(2)

    if "hit" in needs:
        print(f"    hit...", end=" ", flush=True)
        ok = generate_attack_hit(enemy, "hit")
        print("OK" if ok else "FAIL")
        time.sleep(2)

    return True


# ── Main ─────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="PixelLab sprite pipeline")
    parser.add_argument("--test", action="store_true", help="Generate 1 test enemy")
    parser.add_argument("--batch", type=int, help="Generate N enemies")
    parser.add_argument("--all", action="store_true", help="Generate all missing")
    parser.add_argument("--list-missing", action="store_true", help="Show missing sprites")
    parser.add_argument("--regen", action="store_true", help="Regenerate ALL sprites (ignore existing)")
    args = parser.parse_args()

    if not API_KEY:
        print("ERROR: Set PIXELLAB_API_KEY in environment")
        sys.exit(1)

    enemies = parse_manifest()
    print(f"Manifest: {len(enemies)} enemies")

    if args.list_missing or (not args.test and not args.batch and not args.all):
        missing = find_missing(enemies)
        print(f"Missing: {len(missing)} enemies need sprites")
        for e, needs in missing[:20]:
            print(f"  {e['id']}: {', '.join(needs)}")
        if len(missing) > 20:
            print(f"  ... and {len(missing) - 20} more")
        return

    # Prepare style reference
    style_b64 = downscale_to_b64(STYLE_REF, SPRITE_SIZE)

    if args.regen:
        targets = [(e, ["idle", "attack", "hit"]) for e in enemies]
    else:
        targets = find_missing(enemies)

    if args.test:
        targets = targets[:1]
    elif args.batch:
        targets = targets[:args.batch]

    print(f"Generating {len(targets)} enemies ({sum(len(n) for _, n in targets)} sprites)")
    print(f"Estimated generations: {sum(len(n) for _, n in targets)}")

    # Check balance
    try:
        req = urllib.request.Request(
            f"{BASE_URL}/balance",
            headers={"Authorization": f"Bearer {API_KEY}"},
        )
        balance = json.loads(urllib.request.urlopen(req).read())
        gens = balance.get("subscription", {}).get("generations", "?")
        usd = balance.get("credits", {}).get("usd", "?")
        print(f"Balance: {gens} generations, ${usd} credits")
    except Exception:
        pass

    ok_count = 0
    fail_count = 0
    for i, (enemy, needs) in enumerate(targets):
        print(f"\n[{i+1}/{len(targets)}]", end=" ")
        success = generate_enemy(enemy, style_b64, needs)
        if success:
            ok_count += 1
        else:
            fail_count += 1

    print(f"\n\nDone: {ok_count} OK, {fail_count} failed")


if __name__ == "__main__":
    main()
