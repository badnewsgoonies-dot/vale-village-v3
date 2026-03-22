# Booklet Update Changeset — March 22, 2026

## Changes from this session:

### Section 2 — Gemini via Vertex AI
- FIX: Remove "iterate all parts, collect text where thoughtSignature is absent"
- REPLACE WITH: "Just collect text from all parts — don't filter on thoughtSignature. Thinking models put response text in the same part as signature metadata. Filtering it out throws away the answer. Fix was 3 lines replacing 9 (2ba8bed)."
- ADD: `gemini_generate_json` to Python import line
- ADD: "All Vertex models require maxOutputTokens >= 256" (already in some versions)
- ADD: "2.5-flash needs default max_tokens (4096) — 32 is too small for thinking budget"

### Section 4 — INV-032 (upgrade)
- WAS: "(candidate) — Evidence tags anchor relay scope for metadata-responsive families"
- NOW: "Evidence tags anchor relay scope for Claude; checkpointing anchors Gemini."
  - Tags held Claude at observation/principle through 3 passes (B8 + B12)
  - Tags insufficient for Gemini — drifted to law even with tags (B8 + B12-A)
  - Checkpointing (extracting verified claims between passes) held Gemini at principle for all passes (B12-B: 6/6)
  - Cost per checkpoint: 1 Gemini Flash call (~0 premium, ~2s)
  - The defense is pass-level verification, not seed-level tagging

### Section 4 — Five-model relay table
- ADD column: "Checkpointed scope" 
- Gemini Flash: principle×3 (checkpointed) vs principle→principle→law (tagged only)
- Gemini Pro: principle×3 (checkpointed) vs law→principle→law (tagged only)  
- Sonnet: principle→principle→theorem (checkpointed) vs principle→law→principle (tagged only)

### Section 9 — Current State
- VV3 LOC: ~15,778 → 22,348
- VV3 Tests: 232 → 370
- VV3 Domains: 14 → 24
- VV3 Bounded types: 14 → 16
- VV3 Sprites: 268+ → 434/434
- ADD: Adventure mode (--adventure): world map, towns, shops, dialogue, dungeons, quests
- ADD: Content: 10 map nodes, 4 towns, 5 dungeons (24 rooms), 9 dialogue trees, 6 quests

### Section 10 — Trial Protocol
- ADD Battery 12 to completed batteries (18 trials)
- UPDATE total: 1,069 + 18 = 1,087 trials
- ADD to defense map: "Relay scope drift (Gemini) | Checkpointing between passes | B12: 6/6 held at principle | Closed"

### Operational Findings (new)
- Worker fix-loop learning curve: 4 fixes (Batch A) → 3 (B) → 0 (C) → 0 (D)
- Bounded-type constructor warning in dispatch prompts eliminated entire error class
- This is INV-025 in practice: constraint moved from content to interface
