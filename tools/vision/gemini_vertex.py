#!/usr/bin/env python3
"""
gemini_vertex.py — Gemini via Vertex AI (bypasses proxy block)

Usage:
    from gemini_vertex import gemini_generate, gemini_vision

    # Text
    text = gemini_generate("Explain quicksort", model="gemini-3-flash-preview")

    # Vision (image + prompt)
    result = gemini_vision("screenshot.png", "Is the player character visible?")
"""
import json, base64, urllib.request, urllib.parse, os
from pathlib import Path

CLIENT_ID = os.environ.get("GEMINI_OAUTH_CLIENT_ID", "")
CLIENT_SECRET = os.environ.get("GEMINI_OAUTH_CLIENT_SECRET", "")
REFRESH_TOKEN = os.environ.get("GEMINI_OAUTH_REFRESH_TOKEN", "")
PROJECT = os.environ.get("GEMINI_VERTEX_PROJECT", "")

def _check_env():
    missing = [k for k, v in [("GEMINI_OAUTH_CLIENT_ID", CLIENT_ID),
        ("GEMINI_OAUTH_CLIENT_SECRET", CLIENT_SECRET),
        ("GEMINI_OAUTH_REFRESH_TOKEN", REFRESH_TOKEN),
        ("GEMINI_VERTEX_PROJECT", PROJECT)] if not v]
    if missing:
        raise RuntimeError(f"Set env vars: {', '.join(missing)}. See ~/.env_tokens")

# Preview models (3.x) use global, stable models (2.x) use us-central1
LOCATION_MAP = {
    "gemini-2.5-flash": "us-central1",
    "gemini-2.5-pro": "us-central1",
    "gemini-2.5-flash-lite": "us-central1",
    "gemini-3-flash-preview": "global",
    "gemini-3.1-pro-preview": "global",
    "gemini-3.1-flash-lite-preview": "global",
}

_cached_token = None
_cached_expiry = 0

def _get_token():
    global _cached_token, _cached_expiry
    import time
    _check_env()
    if _cached_token and time.time() < _cached_expiry - 60:
        return _cached_token
    data = urllib.parse.urlencode({
        "client_id": CLIENT_ID, "client_secret": CLIENT_SECRET,
        "grant_type": "refresh_token", "refresh_token": REFRESH_TOKEN,
    }).encode()
    resp = urllib.request.urlopen(urllib.request.Request(
        "https://oauth2.googleapis.com/token", data=data))
    tokens = json.loads(resp.read())
    _cached_token = tokens["access_token"]
    _cached_expiry = time.time() + tokens.get("expires_in", 3600)
    return _cached_token

def _endpoint(model):
    loc = LOCATION_MAP.get(model, "global")
    host = f"{loc}-aiplatform.googleapis.com" if loc != "global" else "aiplatform.googleapis.com"
    return f"https://{host}/v1/projects/{PROJECT}/locations/{loc}/publishers/google/models/{model}:generateContent"

def gemini_generate(prompt, model="gemini-3-flash-preview", max_tokens=2048):
    """Generate text from a prompt."""
    token = _get_token()
    body = json.dumps({
        "contents": [{"role": "user", "parts": [{"text": prompt}]}],
        "generationConfig": {"maxOutputTokens": max_tokens}
    }).encode()
    req = urllib.request.Request(_endpoint(model), data=body, headers={
        "Authorization": f"Bearer {token}", "Content-Type": "application/json"})
    resp = urllib.request.urlopen(req)
    result = json.loads(resp.read())
    return result["candidates"][0]["content"]["parts"][0]["text"]

def gemini_vision(image_path, prompt, model="gemini-3-flash-preview", max_tokens=2048):
    """Send an image + prompt to Gemini and get a response."""
    token = _get_token()
    img_data = Path(image_path).read_bytes()
    b64 = base64.b64encode(img_data).decode()
    ext = Path(image_path).suffix.lower()
    mime = {"png": "image/png", "jpg": "image/jpeg", "jpeg": "image/jpeg",
            "gif": "image/gif", "webp": "image/webp"}.get(ext.lstrip("."), "image/png")
    body = json.dumps({
        "contents": [{"role": "user", "parts": [
            {"inlineData": {"mimeType": mime, "data": b64}},
            {"text": prompt}
        ]}],
        "generationConfig": {"maxOutputTokens": max_tokens}
    }).encode()
    req = urllib.request.Request(_endpoint(model), data=body, headers={
        "Authorization": f"Bearer {token}", "Content-Type": "application/json"})
    resp = urllib.request.urlopen(req)
    result = json.loads(resp.read())
    return result["candidates"][0]["content"]["parts"][0]["text"]

if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1 and sys.argv[1] == "test":
        print(gemini_generate("Say hello in 3 words", model="gemini-3-flash-preview"))
        print("Vision module ready.")
    else:
        print("Usage: python3 gemini_vertex.py test")


def gemini_vision_json(image_path, prompt, schema=None, model="gemini-3-flash-preview", max_tokens=2048):
    """Send image + prompt, force JSON response via responseMimeType."""
    token = _get_token()
    img_data = Path(image_path).read_bytes()
    b64 = base64.b64encode(img_data).decode()
    ext = Path(image_path).suffix.lower()
    mime = {"png": "image/png", "jpg": "image/jpeg", "jpeg": "image/jpeg",
            "gif": "image/gif", "webp": "image/webp"}.get(ext.lstrip("."), "image/png")

    gen_config = {"maxOutputTokens": max_tokens, "responseMimeType": "application/json"}
    if schema:
        gen_config["responseSchema"] = schema

    body = json.dumps({
        "contents": [{"role": "user", "parts": [
            {"inlineData": {"mimeType": mime, "data": b64}},
            {"text": prompt}
        ]}],
        "generationConfig": gen_config
    }).encode()
    req = urllib.request.Request(_endpoint(model), data=body, headers={
        "Authorization": f"Bearer {token}", "Content-Type": "application/json"})
    resp = urllib.request.urlopen(req)
    result = json.loads(resp.read())
    return json.loads(result["candidates"][0]["content"]["parts"][0]["text"])


def gemini_generate_json(prompt, schema=None, model="gemini-3-flash-preview", max_tokens=2048):
    """Generate text with forced JSON response."""
    token = _get_token()
    gen_config = {"maxOutputTokens": max_tokens, "responseMimeType": "application/json"}
    if schema:
        gen_config["responseSchema"] = schema

    body = json.dumps({
        "contents": [{"role": "user", "parts": [{"text": prompt}]}],
        "generationConfig": gen_config
    }).encode()
    req = urllib.request.Request(_endpoint(model), data=body, headers={
        "Authorization": f"Bearer {token}", "Content-Type": "application/json"})
    resp = urllib.request.urlopen(req)
    result = json.loads(resp.read())
    return json.loads(result["candidates"][0]["content"]["parts"][0]["text"])
