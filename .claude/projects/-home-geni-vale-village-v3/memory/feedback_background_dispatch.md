---
name: background-dispatch
description: Always launch worker agents in background so orchestrator stays free for other work
type: feedback
---

Launch all worker agents with run_in_background: true so the orchestrator is free meanwhile.

**Why:** User wants to be able to interact / give direction while workers build.

**How to apply:** Every Agent dispatch for domain workers should use run_in_background: true. Check results when notified of completion, then verify + commit + dispatch next.
