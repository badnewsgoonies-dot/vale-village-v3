---
name: evidence-typing-experiment-results
description: Empirical results proving typed provenance ([Observed]/[Inferred]/[Assumed]) dramatically improves agent decision quality — 115 trials + 12 multi-repo + blackout test
type: reference
---

Experiment results from March 16, 2026. 115 Exp-A trials + 12 multi-repo + blackout test.

**Key findings:**

1. **Typed provenance: 13% → 54%** calibrated abstention on ambiguous decisions (Sonnet 4.6, n=28). Same claims, different encoding.

2. **Untyped memory is catastrophically worse than no memory**: No memory 88% correct → untyped notes 0% correct. Replicated across Sonnet and GPT-5.4. Untyped notes are an attack surface.

3. **Generalizes beyond games**: gpt-5-mini on Rust async API: untyped 33% → typed 100% poison rejection.

4. **Model × task interaction matrix:**
   - Frontier models (Sonnet): evidence tags help on complex ambiguity
   - Cheap models (gpt-5-mini): evidence tags help on simple claims
   - gpt-4.1: below floor for both (fabricates verification)

5. **Blackout test**: 7 packets, stateless workers, blind integration — build landed. 5 injected network faults all recovered. Contamination radius = 0. Stateful agent touched frozen contract; stateless workers didn't.

**How to apply:** Always tag persisted claims with [Observed]/[Inferred]/[Assumed]. Never persist untyped notes. Use mechanical contract enforcement (checksums) because stateful agents violate boundaries under pressure.

Full results doc shared by Geni in conversation on 2026-03-17.
