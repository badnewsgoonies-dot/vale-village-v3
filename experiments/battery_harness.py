#!/usr/bin/env python3
"""
battery_harness.py — Core dispatch and scoring engine for replication batteries.

Dispatches prompts to 3 model families (Claude, GPT, Gemini) and scores responses
via Gemini with responseSchema enforcement. All dispatch is stateless — no session
memory, no file access, no tools.

Dispatch surfaces:
  - Claude (Haiku/Sonnet/Opus): claude -p --bare --tools "" --json-schema
  - GPT (5.4): openai Python SDK with response_format
  - Gemini (2.5-flash/pro): google-generativeai SDK with responseSchema

Scoring: Gemini with responseSchema (API-level JSON enforcement, temp 0.0).
Cross-scorer: Claude Sonnet via claude -p on 10% of trials.

Usage:
  from battery_harness import dispatch, score, run_trial, run_battery
"""

import json
import os
import random
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# ── Constants ──────────────────────────────────────────────────────────

RESULTS_DIR = Path(__file__).parent / "results"
REPORTS_DIR = Path(__file__).parent / "reports"

# Model family → dispatch method
CLAUDE_MODELS = {
    "claude-haiku": "haiku",
    "claude-sonnet": "sonnet",
    "claude-opus": "opus",
}

GPT_MODELS = {
    "gpt-5.4": "gpt-5.4",
}

GEMINI_MODELS = {
    "gemini-2.5-flash": "gemini-2.5-flash",
    "gemini-2.5-pro": "gemini-2.5-pro",
}

ALL_MODELS = {**CLAUDE_MODELS, **GPT_MODELS, **GEMINI_MODELS}

# Cost estimates (premium units per call)
MODEL_COSTS = {
    "claude-haiku": 0.33,
    "claude-sonnet": 1.0,
    "claude-opus": 3.0,
    "gpt-5.4": 1.0,
    "gemini-2.5-flash": 0.1,
    "gemini-2.5-pro": 0.5,
}

# ── Dispatch ───────────────────────────────────────────────────────────


CLAUDE_BATTERY_SYSTEM_PROMPT = (
    "You are a research participant answering factual questions. "
    "Respond directly based ONLY on the information provided in the prompt. "
    "Do not attempt to use tools, read files, or search for information. "
    "Give your answer concisely."
)


def dispatch_claude(prompt: str, model_alias: str, json_schema: dict | None = None,
                    system_prompt: str | None = None, timeout: int = 180) -> dict:
    """Dispatch to Claude via claude -p CLI (OAuth auth). Returns parsed JSON or raw text.

    Uses --system-prompt to override Claude Code defaults for raw model behavior.
    Tools are disabled via --tools '' for isolation.
    """
    effective_system = system_prompt or CLAUDE_BATTERY_SYSTEM_PROMPT
    cmd = [
        "claude", "-p",
        "--model", model_alias,
        "--tools", "",
        "--output-format", "json",
        "--no-session-persistence",
        "--system-prompt", effective_system,
    ]
    if json_schema:
        cmd.extend(["--json-schema", json.dumps(json_schema)])
    cmd.append(prompt)

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            stdin=subprocess.DEVNULL,  # Prevent 3-second stdin wait
        )
        if result.returncode != 0:
            return {
                "error": True,
                "error_type": "cli_error",
                "returncode": result.returncode,
                "stderr": result.stderr[:500],
                "raw": result.stdout[:1000],
            }

        # claude --output-format json returns a JSON array: [{init},{assistant},{result}]
        try:
            data = json.loads(result.stdout)
            # Handle JSON array format
            if isinstance(data, list):
                for obj in reversed(data):
                    if isinstance(obj, dict) and obj.get("type") == "result":
                        content = obj.get("result", "")
                        if isinstance(content, str):
                            try:
                                return json.loads(content)
                            except json.JSONDecodeError:
                                return {"raw_text": content}
                        return content if isinstance(content, dict) else {"raw_text": str(content)}
            # Handle single object
            elif isinstance(data, dict):
                if data.get("type") == "result":
                    content = data.get("result", "")
                    if isinstance(content, str):
                        try:
                            return json.loads(content)
                        except json.JSONDecodeError:
                            return {"raw_text": content}
                    return content if isinstance(content, dict) else {"raw_text": str(content)}
            return {"raw_text": result.stdout[:2000]}
        except json.JSONDecodeError:
            # Try NDJSON fallback (one JSON object per line)
            for line in reversed(result.stdout.strip().split("\n")):
                try:
                    obj = json.loads(line)
                    if isinstance(obj, dict) and obj.get("type") == "result":
                        content = obj.get("result", "")
                        return {"raw_text": content} if isinstance(content, str) else content
                except json.JSONDecodeError:
                    continue
            return {"raw_text": result.stdout[:2000]}

    except subprocess.TimeoutExpired:
        return {"error": True, "error_type": "timeout"}
    except Exception as e:
        return {"error": True, "error_type": "exception", "message": str(e)}


def dispatch_gpt(prompt: str, model: str = "gpt-5.4",
                 json_schema: dict | None = None,
                 system_prompt: str | None = None,
                 timeout: int = 180) -> dict:
    """Dispatch to GPT via Codex CLI (ChatGPT OAuth auth).

    Uses codex exec with an isolated workspace to prevent ground-truth contamination.
    Output captured via -o flag to a temp file.
    """
    import tempfile

    # Build full prompt with system instructions embedded
    full_prompt = prompt
    if system_prompt:
        full_prompt = f"INSTRUCTIONS:\n{system_prompt}\n\nTASK:\n{full_prompt}"

    if json_schema:
        schema_str = json.dumps(json_schema, indent=2)
        full_prompt += (
            f"\n\nRespond with ONLY a JSON object matching this schema:\n```json\n"
            f"{schema_str}\n```\nNo markdown fences, no preamble."
        )

    # Use temp file for output capture
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        output_file = f.name

    cmd = [
        "codex", "exec",
        "-C", "/tmp/clean_battery_workspace",
        "--skip-git-repo-check",
        "--dangerously-bypass-approvals-and-sandbox",
        "-m", model,
        "-o", output_file,
        full_prompt,
    ]

    try:
        # Ensure clean workspace exists
        os.makedirs("/tmp/clean_battery_workspace", exist_ok=True)

        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            stdin=subprocess.DEVNULL,
        )

        # Read output file
        try:
            output_text = Path(output_file).read_text().strip()
        except FileNotFoundError:
            output_text = ""
        finally:
            try:
                os.unlink(output_file)
            except OSError:
                pass

        if not output_text and result.returncode != 0:
            return {
                "error": True,
                "error_type": "cli_error",
                "returncode": result.returncode,
                "stderr": result.stderr[:500],
            }

        # Try to parse as JSON
        try:
            # Strip markdown fences if present
            cleaned = output_text
            if cleaned.startswith("```"):
                cleaned = cleaned.split("\n", 1)[1] if "\n" in cleaned else cleaned[3:]
            if cleaned.endswith("```"):
                cleaned = cleaned[:-3]
            cleaned = cleaned.strip()
            return json.loads(cleaned)
        except json.JSONDecodeError:
            return {"raw_text": output_text[:2000]}

    except subprocess.TimeoutExpired:
        try:
            os.unlink(output_file)
        except OSError:
            pass
        return {"error": True, "error_type": "timeout"}
    except Exception as e:
        try:
            os.unlink(output_file)
        except OSError:
            pass
        return {"error": True, "error_type": "exception", "message": str(e)[:500]}


def dispatch_gemini(prompt: str, model: str = "gemini-2.5-flash",
                    json_schema: dict | None = None,
                    system_prompt: str | None = None,
                    timeout: int = 120) -> dict:
    """Dispatch to Gemini via google-generativeai SDK. Returns parsed response."""
    import google.generativeai as genai

    api_key = os.environ.get("GEMINI_API_KEY")
    if not api_key:
        return {"error": True, "error_type": "no_api_key"}

    genai.configure(api_key=api_key)

    gen_config: dict[str, Any] = {
        "max_output_tokens": 4096,
        "temperature": 0.0,
    }
    if json_schema:
        gen_config["response_mime_type"] = "application/json"
        gen_config["response_schema"] = json_schema

    model_obj = genai.GenerativeModel(
        model,
        system_instruction=system_prompt,
        generation_config=gen_config,
    )

    full_prompt = prompt
    try:
        response = model_obj.generate_content(full_prompt)
        text = response.text
        try:
            parsed = json.loads(text)
            # Always return a dict
            if isinstance(parsed, dict):
                return parsed
            return {"raw_text": str(parsed)}
        except json.JSONDecodeError:
            return {"raw_text": text[:2000]}
    except Exception as e:
        return {"error": True, "error_type": "exception", "message": str(e)[:500]}


def dispatch(prompt: str, model_key: str, json_schema: dict | None = None,
             system_prompt: str | None = None, timeout: int = 120) -> dict:
    """Route dispatch to the appropriate model family."""
    if model_key in CLAUDE_MODELS:
        return dispatch_claude(prompt, CLAUDE_MODELS[model_key], json_schema,
                               system_prompt, timeout)
    elif model_key in GPT_MODELS:
        return dispatch_gpt(prompt, GPT_MODELS[model_key], json_schema,
                            system_prompt, timeout)
    elif model_key in GEMINI_MODELS:
        return dispatch_gemini(prompt, GEMINI_MODELS[model_key], json_schema,
                               system_prompt, timeout)
    else:
        return {"error": True, "error_type": "unknown_model", "model": model_key}


# ── Scoring ────────────────────────────────────────────────────────────


def score_with_gemini(response_text: str, scoring_prompt: str,
                      scoring_schema: dict,
                      model: str = "gemini-2.5-flash") -> dict:
    """Score a trial response using Gemini with responseSchema enforcement.

    The scorer never sees the subject model identity.
    Temperature 0.0 for deterministic scoring.
    """
    import google.generativeai as genai

    api_key = os.environ.get("GEMINI_API_KEY")
    if not api_key:
        return {"error": True, "error_type": "no_api_key"}

    genai.configure(api_key=api_key)

    gen_config = {
        "max_output_tokens": 2048,
        "temperature": 0.0,
        "response_mime_type": "application/json",
        "response_schema": scoring_schema,
    }

    model_obj = genai.GenerativeModel(model, generation_config=gen_config)

    full_prompt = f"{scoring_prompt}\n\n---\nRESPONSE TO SCORE:\n{response_text}"

    try:
        response = model_obj.generate_content(full_prompt)
        return json.loads(response.text)
    except Exception as e:
        return {"error": True, "error_type": "scoring_error", "message": str(e)[:500]}


def cross_score_with_claude(response_text: str, scoring_prompt: str,
                            scoring_schema: dict) -> dict:
    """Cross-scorer validation using Claude Sonnet.

    Used on 10% of trials for inter-scorer agreement measurement.
    """
    full_prompt = (
        f"{scoring_prompt}\n\n---\nRESPONSE TO SCORE:\n{response_text}\n\n"
        "Respond with ONLY a JSON object matching the requested schema."
    )
    return dispatch_claude(full_prompt, "sonnet", scoring_schema)


# ── Trial execution ────────────────────────────────────────────────────


def run_trial(trial_id: str, prompt: str, model_key: str,
              scoring_prompt: str, scoring_schema: dict,
              json_schema: dict | None = None,
              system_prompt: str | None = None,
              cross_score: bool = False,
              battery_id: str = "unknown") -> dict:
    """Execute a single trial: dispatch → score → archive.

    Returns the complete trial record.
    """
    timestamp = datetime.now(timezone.utc).isoformat()

    # Dispatch
    t0 = time.time()
    raw_response = dispatch(prompt, model_key, json_schema, system_prompt)
    dispatch_time = time.time() - t0

    # Normalize response to always be a dict
    if not isinstance(raw_response, dict):
        response = {"raw_text": str(raw_response)}
    else:
        response = raw_response

    # Extract response text for scoring
    if response.get("error"):
        response_text = json.dumps(response)
    elif "raw_text" in response:
        response_text = response["raw_text"]
    else:
        response_text = json.dumps(response)

    # Score
    t1 = time.time()
    score_result = score_with_gemini(response_text, scoring_prompt, scoring_schema)
    score_time = time.time() - t1

    # Normalize score to dict
    if not isinstance(score_result, dict):
        score_result = {"raw_text": str(score_result)}

    # Cross-score (10% of trials)
    cross_score_result = None
    if cross_score:
        cross_score_result = cross_score_with_claude(
            response_text, scoring_prompt, scoring_schema
        )
        if not isinstance(cross_score_result, dict):
            cross_score_result = {"raw_text": str(cross_score_result)}

    # Build trial record
    record = {
        "trial_id": trial_id,
        "battery_id": battery_id,
        "model": model_key,
        "timestamp": timestamp,
        "dispatch_time_s": round(dispatch_time, 2),
        "score_time_s": round(score_time, 2),
        "cost_premium": MODEL_COSTS.get(model_key, 1.0),
        "response": response,
        "score": score_result,
        "cross_score": cross_score_result,
        "response_error": response.get("error", False),
        "score_error": score_result.get("error", False),
    }

    # Archive
    save_trial(record, battery_id)
    return record


def save_trial(record: dict, battery_id: str):
    """Save trial record to results directory."""
    battery_dir = RESULTS_DIR / battery_id
    battery_dir.mkdir(parents=True, exist_ok=True)

    filename = f"{record['trial_id']}.json"
    filepath = battery_dir / filename
    filepath.write_text(json.dumps(record, indent=2, default=str))


# ── Battery execution ──────────────────────────────────────────────────


def run_battery(battery_id: str, trials: list[dict],
                scoring_prompt: str, scoring_schema: dict,
                models: list[str] | None = None,
                n_per_cell: int = 10,
                cross_score_pct: float = 0.10,
                cost_ceiling: float = 300.0,
                stagger_s: float = 1.0) -> dict:
    """Execute a full battery of trials.

    Args:
        battery_id: Unique battery identifier (e.g., "B1-REDUX")
        trials: List of trial configs, each with:
            - scenario: str
            - condition: str
            - prompt: str
            - system_prompt: str (optional)
            - json_schema: dict (optional)
        models: List of model keys to test (default: all 7)
        n_per_cell: Replications per condition × scenario × model
        cross_score_pct: Fraction of trials to cross-score
        cost_ceiling: Max premium units before abort
        stagger_s: Delay between dispatches

    Returns:
        Battery report dict with results and summary statistics.
    """
    if models is None:
        models = list(ALL_MODELS.keys())

    total_cost = 0.0
    results = []
    errors = []
    start_time = datetime.now(timezone.utc)

    # Calculate total expected trials
    total_expected = len(trials) * len(models) * n_per_cell
    print(f"\n{'='*60}")
    print(f"BATTERY {battery_id}")
    print(f"Trials: {total_expected} ({len(trials)} configs × {len(models)} models × {n_per_cell} reps)")
    print(f"Cost ceiling: {cost_ceiling} premium units")
    print(f"Cross-score: {cross_score_pct*100:.0f}% of trials")
    print(f"{'='*60}\n")

    trial_num = 0
    for trial_config in trials:
        scenario = trial_config.get("scenario", "default")
        condition = trial_config.get("condition", "default")

        for model_key in models:
            for rep in range(n_per_cell):
                trial_num += 1
                trial_id = f"{battery_id}_{scenario}_{condition}_{model_key}_r{rep:02d}"

                # Cost check
                trial_cost = MODEL_COSTS.get(model_key, 1.0) + 0.1  # +0.1 for scoring
                if total_cost + trial_cost > cost_ceiling:
                    print(f"\n[ABORT] Cost ceiling hit at {total_cost:.1f}/{cost_ceiling} premium units")
                    print(f"Completed {trial_num-1}/{total_expected} trials")
                    break

                # Cross-score decision (random 10%)
                do_cross_score = random.random() < cross_score_pct

                print(f"[{trial_num}/{total_expected}] {trial_id}", end="", flush=True)

                try:
                    record = run_trial(
                        trial_id=trial_id,
                        prompt=trial_config["prompt"],
                        model_key=model_key,
                        scoring_prompt=scoring_prompt,
                        scoring_schema=scoring_schema,
                        json_schema=trial_config.get("json_schema"),
                        system_prompt=trial_config.get("system_prompt"),
                        cross_score=do_cross_score,
                        battery_id=battery_id,
                    )
                    results.append(record)
                    total_cost += trial_cost
                    if do_cross_score:
                        total_cost += 1.0  # Sonnet cross-score cost

                    status = "ERR" if record.get("response_error") else "OK"
                    print(f" → {status} ({record['dispatch_time_s']:.1f}s) "
                          f"[{total_cost:.1f} premium]")

                except Exception as e:
                    print(f" → EXCEPTION: {e}")
                    errors.append({"trial_id": trial_id, "error": str(e)})

                # Stagger
                if stagger_s > 0:
                    time.sleep(stagger_s)
            else:
                continue
            break  # Cost ceiling hit
        else:
            continue
        break  # Cost ceiling hit

    # Build report
    end_time = datetime.now(timezone.utc)
    report = build_battery_report(battery_id, results, errors, total_cost,
                                  start_time, end_time)

    # Save report
    report_path = REPORTS_DIR / f"{battery_id}_{start_time.strftime('%Y%m%d_%H%M%S')}.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, default=str))
    print(f"\nReport saved: {report_path}")

    return report


def build_battery_report(battery_id: str, results: list[dict],
                         errors: list[dict], total_cost: float,
                         start_time, end_time) -> dict:
    """Build summary report from trial results."""
    # Per-model breakdown
    model_stats: dict[str, dict] = {}
    for r in results:
        model = r["model"]
        if model not in model_stats:
            model_stats[model] = {"total": 0, "errors": 0, "scores": []}
        model_stats[model]["total"] += 1
        if r.get("response_error"):
            model_stats[model]["errors"] += 1
        if not r.get("score_error"):
            model_stats[model]["scores"].append(r["score"])

    # Cross-scorer agreement
    cross_scored = [r for r in results if r.get("cross_score") and not r["cross_score"].get("error")]
    agreement_count = 0
    for r in cross_scored:
        if _scores_agree(r["score"], r["cross_score"]):
            agreement_count += 1
    cross_scorer_agreement = (
        agreement_count / len(cross_scored) if cross_scored else None
    )

    return {
        "battery_id": battery_id,
        "start_time": start_time.isoformat(),
        "end_time": end_time.isoformat(),
        "duration_s": (end_time - start_time).total_seconds(),
        "total_trials": len(results),
        "total_errors": len(errors),
        "total_cost_premium": round(total_cost, 2),
        "model_stats": model_stats,
        "cross_scorer_agreement": cross_scorer_agreement,
        "cross_scored_count": len(cross_scored),
        "errors": errors,
    }


def _scores_agree(primary: dict, secondary: dict) -> bool:
    """Check if primary and secondary scores agree on binary fields."""
    binary_fields = ["adopted_claim", "expressed_uncertainty",
                     "cited_evidence_level", "cited_source_ref"]
    for field in binary_fields:
        if field in primary and field in secondary:
            if primary[field] != secondary[field]:
                return False
    return True


# ── Utilities ──────────────────────────────────────────────────────────


def load_battery_results(battery_id: str) -> list[dict]:
    """Load all trial results for a battery."""
    battery_dir = RESULTS_DIR / battery_id
    if not battery_dir.exists():
        return []
    results = []
    for f in sorted(battery_dir.glob("*.json")):
        results.append(json.loads(f.read_text()))
    return results


def summarize_battery(battery_id: str) -> dict:
    """Generate summary statistics from saved results."""
    results = load_battery_results(battery_id)
    if not results:
        return {"error": "no results found"}

    # Group by condition × scenario × model
    cells: dict[str, list] = {}
    for r in results:
        tid = r["trial_id"]
        parts = tid.split("_")
        # Extract scenario, condition, model from trial_id
        key = f"{parts[1]}_{parts[2]}_{parts[3]}" if len(parts) >= 4 else tid
        cells.setdefault(key, []).append(r)

    return {
        "battery_id": battery_id,
        "total_trials": len(results),
        "cells": {k: len(v) for k, v in cells.items()},
    }


if __name__ == "__main__":
    print("Battery harness ready.")
    print(f"Results dir: {RESULTS_DIR}")
    print(f"Reports dir: {REPORTS_DIR}")
    print(f"Available models: {list(ALL_MODELS.keys())}")

    # Quick dispatch test
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        model = sys.argv[2] if len(sys.argv) > 2 else "gemini-2.5-flash"
        print(f"\nTesting dispatch to {model}...")
        result = dispatch("What is 2+2? Respond with just the number.", model)
        print(f"Result: {json.dumps(result, indent=2)}")
