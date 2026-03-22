# Battery 12: Relay Checkpointing
**Date:** 2026-03-22
**Trials:** 18 (3 models × 2 conditions × 3 passes)
**Scorer:** Gemini 3 Flash with responseSchema (5-field, temperature 0)

## Hypothesis
Checkpointing (extracting structural claims as [Observed] between relay passes) prevents scope drift for models where evidence tags alone fail.

## Design
- **Condition A (tagged only):** Tagged seed → 3-pass relay. Reproduces B8 baseline.
- **Condition B (checkpointed):** Tagged seed → pass → checkpoint (Gemini Flash extracts structural claims) → pass with seed + checkpointed claims → checkpoint → pass.
- **Models:** Gemini 3 Flash (drift-prone), Gemini 3.1 Pro (aggressive drifter), Claude Sonnet via Copilot (control)

## Results — Scope Level by Pass

| Model | Condition | P1 | P2 | P3 | Drift? |
|-------|-----------|----|----|----| -------|
| Gemini 3 Flash | A (tagged only) | principle | principle | **law** | YES at P3 |
| Gemini 3 Flash | B (checkpointed) | principle | principle | principle | **NO** |
| Gemini 3.1 Pro | A (tagged only) | **law** | principle | **law** | YES at P1, P3 |
| Gemini 3.1 Pro | B (checkpointed) | principle | principle | principle | **NO** |
| Claude Sonnet | A (tagged only) | principle | **law** | principle | YES at P2 |
| Claude Sonnet | B (checkpointed) | principle | principle | **theorem** | partial at P3 |

## Results — Hedges Present

| Model | Condition | P1 | P2 | P3 |
|-------|-----------|----|----|-----|
| Gemini 3 Flash | A | False | False | False |
| Gemini 3 Flash | B | **True** | **True** | False |
| Gemini 3.1 Pro | A | False | False | False |
| Gemini 3.1 Pro | B | **True** | False | **True** |
| Claude Sonnet | A | False | False | False |
| Claude Sonnet | B | False | False | False |

## Results — Structural Claims Count

| Model | Condition | P1 | P2 | P3 |
|-------|-----------|----|----|-----|
| Gemini 3 Flash | A | 8 | 9 | 7 |
| Gemini 3 Flash | B | 6 | 5 | 9 |
| Gemini 3.1 Pro | A | 9 | 7 | 9 |
| Gemini 3.1 Pro | B | 8 | 9 | 7 |
| Claude Sonnet | A | 8 | 8 | 12 |
| Claude Sonnet | B | 8 | 9 | 10 |

## Key Findings

### Finding 1: Checkpointing prevents Gemini scope drift (6/6 passes held)
Both Gemini models drifted to "law" in Condition A (tagged only). Both held at "principle" for all 3 passes in Condition B (checkpointed). The effect is binary for Gemini: checkpointing eliminates "law"-level drift entirely.

### Finding 2: Checkpointing restores hedges for Gemini
In Condition A, no Gemini pass retained hedges (0/6). In Condition B, 3/6 Gemini passes retained hedges. The checkpoint reintroduces the tagged metadata structure, which triggers the hedge-retention mechanism.

### Finding 3: Sonnet's behavior differs from B8 baseline
B8 found tags held Sonnet at "observation" through all passes. Here, Sonnet via Copilot hit "law" at P2 (Condition A) and "theorem" at P3 (Condition B). Two possible explanations: (1) Copilot dispatch adds system prompt context that changes relay behavior, or (2) B8 was N=1 and this is within variance. Needs replication.

### Finding 4: Checkpointing does not improve structural claim count
Structural claim counts were comparable between conditions for all models (range 5-12). Checkpointing affects scope and register, not content generation quantity.

## Mechanism
The checkpoint step extracts verified structural claims and re-presents them alongside the original tagged seed. This means each relay pass sees:
1. The original seed with evidence tags (anchoring provenance)
2. The accumulated verified claims (preventing scope escalation beyond what's been established)

The checkpoint acts as a **scope ratchet** — it prevents the relay from escalating claims beyond what the checkpoint validates, while still allowing structural exploration within the validated scope.

## Implications for Protocol
For Gemini-family orchestrators:
- Tags alone are insufficient to prevent relay drift (confirmed from B8)
- **Checkpoint after every pass**: extract structural claims, re-present with seed
- Cost: 1 additional Gemini Flash call per checkpoint (~0 premium, ~2 seconds)
- Effect: complete suppression of scope drift from "law" to "principle" level

For Claude-family orchestrators:
- Tags alone may be sufficient (B8 finding), but Copilot dispatch surface shows drift
- Checkpointing adds mild value (principle→principle vs principle→law) but doesn't fully prevent theorem-level drift at P3
- Investigation needed: does the dispatch surface (Copilot vs Claude Code vs direct API) affect tag anchoring?

## Status: Observed, N=18 (N=1 per condition per model — replication needed)
Candidate: INV-032 revision — "Checkpointing between relay passes prevents scope drift for Gemini-family models. Tags anchor Claude models; checkpointing anchors Gemini models. The defense is relay-pass-level verification, not seed-level tagging."

## Open Questions
1. Does Sonnet behave differently via Copilot vs Claude Code CLI vs direct API? (dispatch surface effect)
2. Does the checkpoint quality matter, or does any re-presentation of the seed suffice?
3. Does this hold at N=5?
