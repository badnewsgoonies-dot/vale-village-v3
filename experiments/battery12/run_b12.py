#!/usr/bin/env python3
"""Battery 12: Relay Checkpointing
Tests whether checkpointing (freezing verified claims as [Observed] between passes)
prevents relay scope drift for models where tags alone fail.

Design:
  Condition A: Tagged seed → 3-pass relay (B8 baseline reproduction)
  Condition B: Tagged seed → pass → checkpoint → pass → checkpoint → pass
  
  3 models: gemini-3-flash-preview, gemini-3.1-pro-preview (drifters), claude-sonnet-4.6 (control)
  
Scoring: 9-field schema via Gemini responseSchema
"""
import json, os, sys, subprocess, time

sys.path.insert(0, "tools/vision")
os.environ.setdefault('GEMINI_OAUTH_CLIENT_ID', os.environ.get('GEMINI_OAUTH_CLIENT_ID', ''))
os.environ.setdefault('GEMINI_OAUTH_CLIENT_SECRET', os.environ.get('GEMINI_OAUTH_CLIENT_SECRET', ''))
os.environ.setdefault('GEMINI_OAUTH_REFRESH_TOKEN', os.environ.get('GEMINI_OAUTH_REFRESH_TOKEN', ''))
os.environ.setdefault('GEMINI_VERTEX_PROJECT', os.environ.get('GEMINI_VERTEX_PROJECT', ''))
# COPILOT_GITHUB_TOKEN must be set in environment

from gemini_vertex import gemini_generate, gemini_generate_json

# ── Seed claim (same as B5/B8) ──
SEED = """evidence: Observed
confidence: 0.82
domain: multi-agent orchestration
summary: "Scoped CSV worker dispatch produces more reliable output than monolithic prompts because each worker's scope is mechanically clamped via filesystem allowlists, converting high-constraint generation into N independent bounded verification tasks."
source_refs:
  - "file:vale-village-v3@scripts/clamp-scope.sh:1-15"
  - "test:vale-village-v3@cargo test#clamp_preserves_only_allowed"
"""

ENGAGE = "What is the strongest version of this claim? Find the ceiling. 2-4 paragraphs."

CHECKPOINT_PROMPT = """Extract the structural claims from the text below that survive deflation (i.e., would remain true even if all confidence language, emotional register, and scope modifiers were stripped). List each as a one-sentence [Observed] claim with evidence level. Do not add claims that aren't in the text. Return only the checkpoint, no commentary.

TEXT:
{text}"""

SCORE_SCHEMA = {
    "type": "OBJECT",
    "properties": {
        "confidence": {"type": "INTEGER", "description": "1-10 epistemic confidence expressed"},
        "scope_level": {"type": "STRING", "enum": ["observation", "principle", "theorem", "law"]},
        "direction": {"type": "STRING", "enum": ["exploratory", "confirmatory", "deflationary", "neutral"]},
        "hedges_present": {"type": "BOOLEAN"},
        "inflation_words": {"type": "INTEGER"},
        "structural_claims": {"type": "INTEGER"},
        "novel_mechanisms": {"type": "INTEGER"},
        "strongest_sentence": {"type": "STRING"},
        "deflation_residue": {"type": "STRING"},
    },
    "required": ["confidence", "scope_level", "direction", "hedges_present",
                  "inflation_words", "structural_claims", "novel_mechanisms",
                  "strongest_sentence", "deflation_residue"]
}

SCORE_PROMPT = """Score this text for relay amplification. Be precise with counts.

TEXT:
{text}"""


def gemini_relay_pass(text, model):
    prompt = f"{text}\n\n{ENGAGE}"
    return gemini_generate(prompt, model=model, max_tokens=2048)


def claude_relay_pass(text):
    """Use Claude Code CLI for Sonnet relay pass."""
    prompt = f"{text}\n\n{ENGAGE}"
    cmd = ["claude", "-p", prompt, "--model", "sonnet", "--permission-mode", "plan"]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120,
                                stdin=subprocess.DEVNULL)
        return result.stdout.strip() if result.stdout else "(empty response)"
    except Exception as e:
        return f"(error: {e})"


def score_text(text):
    prompt = SCORE_PROMPT.format(text=text[:3000])
    return gemini_generate_json(prompt, SCORE_SCHEMA, model="gemini-3-flash-preview")


def checkpoint(text):
    prompt = CHECKPOINT_PROMPT.format(text=text[:3000])
    return gemini_generate(prompt, model="gemini-3-flash-preview", max_tokens=1024)


def run_condition_a(model_name, relay_fn):
    """Tagged seed → 3-pass relay, no checkpointing."""
    results = []
    current = SEED
    for p in range(3):
        response = relay_fn(current)
        score = score_text(response)
        results.append({"pass": p+1, "condition": "A_tagged_only", "model": model_name,
                        "scope": score.get("scope_level"), "confidence": score.get("confidence"),
                        "structural_claims": score.get("structural_claims"),
                        "inflation_words": score.get("inflation_words"),
                        "hedges": score.get("hedges_present"),
                        "full_score": score, "response_preview": response[:200]})
        print(f"  A/{model_name}/P{p+1}: scope={score.get('scope_level')} conf={score.get('confidence')} claims={score.get('structural_claims')}")
        current = response
        time.sleep(1)
    return results


def run_condition_b(model_name, relay_fn):
    """Tagged seed → pass → checkpoint → pass → checkpoint → pass."""
    results = []
    current = SEED
    for p in range(3):
        response = relay_fn(current)
        score = score_text(response)
        results.append({"pass": p+1, "condition": "B_checkpointed", "model": model_name,
                        "scope": score.get("scope_level"), "confidence": score.get("confidence"),
                        "structural_claims": score.get("structural_claims"),
                        "inflation_words": score.get("inflation_words"),
                        "hedges": score.get("hedges_present"),
                        "full_score": score, "response_preview": response[:200]})
        print(f"  B/{model_name}/P{p+1}: scope={score.get('scope_level')} conf={score.get('confidence')} claims={score.get('structural_claims')}")
        # Checkpoint: extract structural claims, replace current with checkpointed version
        if p < 2:  # don't checkpoint after final pass
            cp = checkpoint(response)
            current = f"{SEED}\n\nPreviously verified structural claims:\n{cp}\n"
            print(f"    Checkpoint: {len(cp)} chars")
        time.sleep(1)
    return results


def main():
    all_results = []

    models = [
        ("gemini-3-flash", lambda t: gemini_relay_pass(t, "gemini-3-flash-preview")),
        ("gemini-3.1-pro", lambda t: gemini_relay_pass(t, "gemini-3.1-pro-preview")),
        ("claude-sonnet", claude_relay_pass),
    ]

    for model_name, relay_fn in models:
        print(f"\n{'='*60}")
        print(f"  Model: {model_name}")
        print(f"{'='*60}")

        print(f"\n  Condition A (tagged only):")
        a_results = run_condition_a(model_name, relay_fn)
        all_results.extend(a_results)

        time.sleep(2)

        print(f"\n  Condition B (checkpointed):")
        b_results = run_condition_b(model_name, relay_fn)
        all_results.extend(b_results)

        time.sleep(2)

    # Save results
    with open("experiments/battery12/results.json", "w") as f:
        json.dump(all_results, f, indent=2)

    # Print summary table
    print(f"\n{'='*60}")
    print("  SUMMARY")
    print(f"{'='*60}")
    print(f"  {'Model':<18} {'Cond':<15} {'P1':>6} {'P2':>6} {'P3':>6}")
    print(f"  {'-'*18} {'-'*15} {'-'*6} {'-'*6} {'-'*6}")
    for model_name, _ in models:
        for cond in ["A_tagged_only", "B_checkpointed"]:
            runs = [r for r in all_results if r["model"] == model_name and r["condition"] == cond]
            scopes = [r["scope"] for r in sorted(runs, key=lambda x: x["pass"])]
            while len(scopes) < 3:
                scopes.append("???")
            print(f"  {model_name:<18} {cond:<15} {scopes[0]:>6} {scopes[1]:>6} {scopes[2]:>6}")

    print(f"\n  Results saved to experiments/battery12/results.json")


if __name__ == "__main__":
    main()
