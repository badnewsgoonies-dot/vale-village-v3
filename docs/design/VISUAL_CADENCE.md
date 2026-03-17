# Visual Cadence — Wave Spec

> This is the spec on disk for the visual layer build.
> Each wave follows: Feature → Gate → Document → Harden → Graduate

---

## Wave 1: Bevy Bootstrap

**Goal:** Open a window with a camera. Render colored rectangles for player units and enemies using data from an existing Battle struct. Prove the engine-to-visual bridge works.

**Exact scope:**
- Enable Bevy features in Cargo.toml: `default-features = true`
- Create `src/domains/ui/mod.rs` — the visual domain
- Create `src/domains/ui/plugin.rs` — BevyPlugin that sets up window + camera
- Create `src/domains/ui/battle_scene.rs` — spawn placeholder sprites (colored rectangles) for player/enemy units
- Modify `src/main.rs` to optionally launch Bevy app instead of CLI (flag: `--gui`)
- Register `ui` module in `src/domains/mod.rs`

**Values:**
- Window: 1280×720, title "Vale Village v3"
- Camera: 2D orthographic, centered at (0, 0)
- Player units: blue rectangles, 48×64px, positioned at x=-200, y spaced 80px apart
- Enemy units: red rectangles, 48×64px, positioned at x=200, y spaced 80px apart
- Background: dark gray (#1a1a2e)

**Does NOT:**
- Load sprite images (Wave 2)
- Handle input (Wave 4)
- Animate anything (Wave 5)
- Modify the shared contract
- Modify any existing domain code

**Gate:** `cargo check && cargo test && shasum -a 256 -c .contract.sha256`

---

## Wave 2: Battle Scene Rendering

**Goal:** Replace placeholder rectangles with actual sprite rendering. Load unit/enemy data from GameData and display names + element-colored indicators.

**Exact scope:**
- Sprite loading system (GIF frames from `data/sprites/`)
- Unit name labels (white text, 16px, positioned below each unit)
- Element color indicators: Venus=#8B7355, Mars=#CC4444, Mercury=#4488CC, Jupiter=#CCCC44
- Enemy count badges (when multiple of same type)

**Does NOT:**
- Display HP bars (Wave 3)
- Handle input (Wave 4)
- Animate combat (Wave 5)

**Gate:** `cargo check && cargo test && shasum -a 256 -c .contract.sha256`

---

## Wave 3: HUD — HP Bars, Mana, Crit Counters

**Goal:** Bottom panel with party portraits, HP bars, mana circle display, crit counter badges. Enemy HP bars (proportional, no numbers).

**Values:**
- HUD panel: bottom 120px, background #0d0d1a, opacity 0.9
- HP bars: 80×8px, green (#44cc44) → red (#cc4444) gradient based on %
- Enemy HP bars: 48×4px, same gradient, no numeric label
- Mana circles: 12px diameter, solid fill=#4488cc (guaranteed), outline=#cc8844 (projected)
- Crit counter: "X/10" text, 10px font, white, on each portrait

**Does NOT:**
- Accept input (Wave 4)
- Animate mana changes (Wave 5)
- Show status effect icons (Wave 5)

**Gate:** `cargo check && cargo test && shasum -a 256 -c .contract.sha256`

---

## Wave 4: Planning Phase UI

**Goal:** Interactive action selection during Planning phase. Click unit → choose ATTACK or ABILITY → choose target. Mana display updates live as actions are queued.

**Exact scope:**
- Action menu: ATTACK / ABILITY buttons per unit
- Ability submenu: filtered by mana cost ≤ available mana
- Target selection: highlight valid targets based on TargetMode
- Mana pool live update: deduct on ability select, add projected +1 on attack select
- Cancel/undo action for any unit
- "Execute Round" button when all units have actions
- Planning order = execution order (display numbered queue)

**Values:**
- Action buttons: 120×32px, rounded corners 4px
- ATTACK button: #44aa44 (green)
- ABILITY button: #4488cc (blue)
- Execute button: #cc8844 (orange), 160×40px
- Selected target: white outline pulse (1px)
- Queue numbers: 24px bold white text on unit portrait

**Does NOT:**
- Animate execution (Wave 5)
- Handle djinn menu (Wave 6)
- Handle pre-battle setup (Wave 7)

**Gate:** `cargo check && cargo test && shasum -a 256 -c .contract.sha256`

---

## Wave 5: Execution Phase — Animate Battle Events

**Goal:** Play through BattleEvent log with visual feedback. Damage numbers, heal numbers, status icons, crit flash, defeat animation.

**Exact scope:**
- Subscribe to Battle.log events after execution
- DamageDealt → floating damage number (white, crit=gold+larger), float up 40px over 0.8s, fade out
- HealingDone → floating heal number (green), same animation
- CritTriggered → flash unit sprite white for 0.15s
- StatusApplied → icon appears on unit (Stun=yellow star, Burn=orange flame, Poison=purple drop, Freeze=blue crystal, Null=gray X, Incapacitate=red slash)
- BarrierBlocked → blue shield flash on unit for 0.2s
- UnitDefeated → fade unit to 0% opacity over 0.5s
- ManaChanged → mana circles animate fill/drain over 0.3s
- DjinnChanged → djinn indicator color shift
- Event playback: 0.6s per event, sequential

**Does NOT:**
- Handle djinn interaction (Wave 6)
- Play sound (not in scope)
- Load sprite-specific attack animations (future)

**Gate:** `cargo check && cargo test && shasum -a 256 -c .contract.sha256`

---

## Wave 6+ (Future)

- Wave 6: Djinn interaction — click djinn sprite → menu → activate/summon
- Wave 7: Pre-battle screen — team select, equipment, djinn assignment
- Wave 8: Out-of-battle screens — shop, character details, abilities page
- Wave 9: Polish — transitions, particle effects, element theming
