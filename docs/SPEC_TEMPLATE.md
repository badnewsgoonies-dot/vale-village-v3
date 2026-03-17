# Project Specification Template

> Copy this file to `docs/spec.md` and fill in every section. Workers read this from disk.
> If a value matters, write the literal number. Agents do not infer values from context.

---

# [Project Name] — Specification

> Single source of truth. When this file contradicts any other document, this file wins.

---

## 1. One-Sentence Pitch

[What this software does in one sentence. Be specific about the mechanism, not the theme.]

Example: "A deterministic queue-based combat engine where ATTACK generates mana and crit, ABILITY spends them, and djinn activation temporarily rewires each unit's available kit."

Example: "A REST API that ingests webhook events from Stripe, validates signatures, normalizes the payload, and writes idempotent records to Postgres."

---

## 2. Core Pillars (3 max)

1. [Pillar] — [one sentence explaining what this means in practice]
2. [Pillar] — [one sentence]
3. [Pillar] — [one sentence]

Pillars are design constraints, not features. "Deterministic combat — no randomness anywhere" constrains every system. "Has a shop" does not.

---

## 3. System Rules

> Every rule needs: the formula or logic, all constants with literal values, and edge cases.

### 3.1 [System Name]

**Formula:** `result = baseValue + modifier - (resistance × 0.5)`, floor 1

**Constants:**
- `BASE_RATE = 10`
- `MAX_LEVEL = 20`
- `MULTIPLIER = 2.0`

**Edge cases:**
- If target is already dead, skip
- If result would exceed max, clamp to max

### 3.2 [Next System]

[Repeat for every system. Include tables for anything with more than 3 variants.]

| Status | Effect | Duration | Mechanic |
|--------|--------|----------|----------|
| Burn | 10% maxHP/turn | 3 turns | Deterministic tick |
| Freeze | Skip turn | Until 40 cumulative damage | Threshold-based break |

---

## 4. Domain Boundaries

> Each domain is a folder. Domains import from the shared contract, not from each other.

| Domain | Path | Owns | Does NOT Own |
|--------|------|------|-------------|
| [name] | `src/domains/[name]/` | [what it handles] | [what it explicitly does NOT handle] |

---

## 5. Content Quantities

> Exact counts. Not "several" or "many." If you write "lots of items," the agent will produce 5.

| Content | Count | Notes |
|---------|-------|-------|
| [Items] | [exact number] | [any constraints] |
| [Enemies] | [exact number] | [level range, element distribution] |
| [Abilities] | [exact number] | [type breakdown] |

---

## 6. Data Schema

> Types that appear in the shared contract. All domains import these — no local redefinitions.

```text
[Language-appropriate type definitions]

Example (Rust):
pub enum Element { Venus, Mars, Mercury, Jupiter }
pub struct Stats { pub hp: u16, pub atk: u16, pub def: u16, pub mag: u16, pub spd: u16 }

Example (TypeScript):
type Element = 'Venus' | 'Mars' | 'Mercury' | 'Jupiter';
interface Stats { hp: number; atk: number; def: number; mag: number; spd: number; }
```

**Primitive decisions (frozen):**
- IDs are [string / number / UUID]
- Timestamps are [ISO 8601 / Unix epoch / etc.]
- Money is [cents as integer / decimal / etc.]

---

## 7. What This Spec Does NOT Cover

- [Thing that might seem in scope but isn't]
- [Thing the agent might try to build but shouldn't]
- [Future feature that should not be started yet]

---

## 8. Definition of Done

The project is done when:

- [ ] [Gate 1 — e.g., "cargo test passes with 0 failures"]
- [ ] [Gate 2 — e.g., "all API endpoints return correct status codes"]
- [ ] [Gate 3 — e.g., "every domain imports from shared contract"]
- [ ] [Surface 1 — e.g., "a user can sign up, log in, and see their dashboard"]
- [ ] [Surface 2 — e.g., "the battle runs from start to victory with visible feedback"]
