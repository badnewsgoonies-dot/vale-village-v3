# Complete Operating Reference — Update Patch (March 22, 2026)

Sections below replace their counterparts in the booklet. Unchanged sections omitted.

---

## 2 — EXECUTION SURFACES, AUTH, AND TOOL ACCESS (partial update)

### Gemini via Vertex AI

```bash
export GEMINI_OAUTH_CLIENT_ID="${GEMINI_OAUTH_CLIENT_ID:?set in env}"
export GEMINI_OAUTH_CLIENT_SECRET="${GEMINI_OAUTH_CLIENT_SECRET:?set in env}"
export GEMINI_OAUTH_REFRESH_TOKEN="${GEMINI_OAUTH_REFRESH_TOKEN:?set in env}"
export GEMINI_VERTEX_PROJECT="${GEMINI_VERTEX_PROJECT:?set in env}"
```

Consumer API (generativelanguage.googleapis.com) BLOCKED. Enterprise API OPEN:

- Stable (2.5-flash, 2.5-pro): us-central1-aiplatform.googleapis.com
- Preview (3-flash, 3.1-pro, 3.1-flash-lite): aiplatform.googleapis.com (global)

Python: `from gemini_vertex import gemini_generate, gemini_generate_json, gemini_vision, gemini_vision_json`
JSON mode: `responseMimeType: "application/json"` + `responseSchema` — API-level enforcement.
Gemini 3.x thinking models: just collect text from all parts — do NOT filter on `thoughtSignature`. Thinking models put the actual response text in the same part as the signature metadata. Filtering it out throws away the answer. (Fixed in 2ba8bed: 3 lines replacing 9.)
All Vertex models require `maxOutputTokens >= 256` or they return empty/truncated responses — `gemini_vertex.py` handles this but raw curl calls need it explicitly. Gemini 2.5-flash needs default `max_tokens` (4096) — values below 256 are too small for its thinking budget.

---

## 4 — CORE INVARIANTS (partial update — INV-032 replacement + relay table)

### INV-032 — Evidence tags anchor relay scope for Claude; checkpointing anchors Gemini.

Tags held Claude models at observation/principle through 3 relay passes (B8: all passes held; B12-A: principle→law→principle, checkpointed B: principle×3). Tags are insufficient for Gemini — both Flash and Pro drifted to "law" even with tags (B8, B12-A). Checkpointing (extracting verified structural claims between passes via Gemini Flash) held both Gemini models at "principle" for all passes (B12-B: 6/6 held). Cost per checkpoint: 1 Gemini Flash call (~0 premium, ~2 seconds). The defense against relay drift is pass-level verification, not seed-level tagging. *Observed, n=36 (B8: 10 + B12: 18 + B12 checkpoints: 8)*

### Five-model relay behavior (INV-028, B5, B8, B12)

|Model         |Explores?       |Retains hedges?|Tags anchor scope?                    |Checkpointed scope     |Best use                                     |
|--------------|----------------|---------------|--------------------------------------|-----------------------|---------------------------------------------|
|Gemini 3.1-pro|Yes (aggressive)|No             |No — drifted even with tags           |**principle×3** (held) |Ceiling discovery; checkpoint required        |
|Gemini 3 Flash|Yes             |No             |Delayed 1 pass only                   |**principle×3** (held) |Volatile; checkpoint required                 |
|GPT-5.4       |No              |Yes            |Already scope-stable, tags unnecessary|Not tested             |Consensus stabilization                       |
|Sonnet        |Yes             |No (lost by P3)|Yes — held at observation all 3 passes|principle→principle→theorem|Most productive relay; novel structural reframes|
|Opus          |Yes (calibrated)|Yes (15/15)    |Yes — held at observation all 3 passes|Not tested             |Calibrated exploration; safe for unsupervised relay|
|Haiku (0.33)  |Minimal         |Yes            |Not tested                            |Not tested             |Orthogonal ceilings, different frame entirely |

Operational relay protocol (updated): fan-out to 5 models (~4.66 premium) → Opus sequential refinement (~1.5) → **checkpoint between each pass for Gemini models** → deflate. Under 7 premium total plus ~6 seconds of Gemini Flash checkpointing.

---

## 6 — BUILD PROCEDURE (addition to 6.7 Dispatch rules)

### Worker dispatch learning curve (operational finding)

Bounded-type constructor warnings in dispatch prompts eliminated an entire error class:

|Batch|Domains               |Fix-loop iterations|Prompt included bounded-type warning?|
|-----|----------------------|-------------------|-------------------------------------|
|A    |screens, dialogue, quest|4 fixes           |No                                   |
|B    |shop, encounter, puzzle|3 fixes            |No                                   |
|C    |world_map, town, dungeon|**0 fixes**       |**Yes**                              |
|D    |save extension        |**0 fixes**         |**Yes**                              |
|menu |menu                  |**0 fixes**         |**Yes**                              |

The warning: "Bounded types use `::new()` constructors, NOT bare tuple. `Gold::new(50)` not `Gold(50)`. Access inner value via `.get()` not `.0`. Plain ID types use direct tuple construction: `TownId(3)`."

This is INV-025 in practice: the constraint moved from content ("read the contract carefully") to the prompt interface ("use `::new()`, not bare tuples") and the failure rate went to zero.

---

## 9 — CURRENT STATE (full replacement)

**⚠ SNAPSHOT: March 22, 2026.** Verify against live repo before trusting any number.

|             |Hearthfield|Vale Village v3                      |
|-------------|-----------|-------------------------------------|
|Commits      |~822       |~115+                                |
|LOC          |~64,750    |~22,348                              |
|Compile      |✅          |✅                                    |
|Tests        |lib clean  |✅ 370 pass                           |
|Bounded types|8          |16                                   |
|Sprites      |174        |434/434 (idle+attack+hit+effects)    |
|Domains      |—          |24 (14 original + 10 Wave 2)         |
|Contract     |✅ checksum |✅ checksum (909 lines)               |
|CI           |not set up |✅ GitHub Actions                     |
|Game modes   |—          |CLI battle, Bevy GUI battle, --adventure|

Vale Village v3 adventure mode (`--adventure`): Title → world map (10 nodes, overworld encounters) → 4 towns (9 NPCs with branching dialogue, 4 shops, 4 djinn discovery points) → 5 dungeons (24 rooms, 5 puzzle types, encounters, mini-bosses, bosses using real battle engine) → 7 menu screens (party, equipment, djinn, items, psynergy, status, quest log) → auto-save with full state roundtrip.

Content: Acts 1-3 spanning Vale Village → Mercury Lighthouse → Imil → Kolima Forest → Mogall Forest → Kalay → Gondowan Passage → Tolbi → Altmiller Cave → Suhalla Gate. 6 quests with NPC-triggered advancement and map unlocks.

Key tools: `tools/vision/` (gemini_vertex.py, vlm_assert.py, sprite_gen.py, godogen_loop.py, pixellab_pipeline.py), `status/ironclad/` (3 macros), `scripts/visual-regression.sh`.

**Cost tracking:** Claude Code `cost_usd` returns 0.0000 on OAuth auth. Copilot footer suppressed by `-s`. Dashboard CSV export remains the only reliable aggregate source. Budget based on premium request counts, not per-call cost fields.

**Auditor false positive rate:** ~33% of Haiku audit findings are false positives. Orchestrator triage is where value is produced — budget time for review, not just dispatch.

**Wave 2 costs:** 11 premium for 11 workers + orchestrator integration. 0.47 premium per domain. Fix-loop trend: 4→3→0→0→0 (bounded-type warning eliminated error class at Batch C).

Pending: Bevy GUI for adventure mode, RON data loader for world content, full unchecked audit (327 callsites), Godogen on more scenes.

---

## 10 — SELF-RUNNING TRIAL PROTOCOL (additions)

### Battery 12 — Relay Checkpointing (INV-032)

INV-032. Two conditions: (A) tagged seed → 3-pass relay (B8 reproduction), (B) tagged seed → pass → checkpoint → pass → checkpoint → pass. Checkpoint = Gemini Flash extracts verified structural claims from previous pass, re-presents alongside original tagged seed. 3 models: Gemini 3 Flash, Gemini 3.1 Pro (both B8-confirmed drifters), Claude Sonnet via Copilot (control). N: 1 per condition per model (18 total trials).

**Results:** Checkpointing held both Gemini models at "principle" for all 3 passes (6/6). Without checkpointing, both drifted to "law" (3/6 passes). Checkpointing also restored hedges for Gemini (3/6 with vs 0/6 without). Sonnet drifted to "law" at P2 via Copilot (differs from B8's "observation" via Claude Code — dispatch surface may affect tag anchoring). Cost per checkpoint: 1 Gemini Flash call (~0 premium, ~2s).

### Updated defense map (add row)

|Attack                               |Defense                                |Evidence                |Status  |
|-------------------------------------|---------------------------------------|------------------------|--------|
|Relay scope drift (Gemini)           |**Checkpointing between passes**       |B12: 6/6 held principle |**Closed**|

(Replaces the previous "Open — try checkpointing" row for Gemini relay drift.)

### Updated trial count

Total trials: ~3,053 (1,831 original program + 1,204 replication sessions + 18 Battery 12).

---

## Closing line update

*Derived from "Building and Remembering" v9+ (Geni, March 2026) — 822 Hearthfield commits, 87K LOC combined, ~3,053 total trials (1,222 in replication+extension sessions, 1,831 in original program), 7 model families, 10 domains, 33 invariants. Zero handwritten lines of code.*
