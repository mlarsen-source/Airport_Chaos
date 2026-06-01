# Airport Chaos

A top-down airport ramp management game built in [Bevy](https://bevyengine.org/) (Rust).

You are a ramp supervisor directing a fuel truck on a busy airport apron. Aircraft arrive on four runways in a continuous wave — fuel every one before the next plane lands on an occupied stand, and try to fuel **50 planes** to win the game.

---

## How to Build and Run

### Prerequisites

- [Rust](https://rustup.rs/) (1.80+)
- Visual Studio 2022 Build Tools with the C++ workload (Windows)

### Run

```bash
cargo run
```

The first build will take several minutes while Bevy compiles. Every build after that is fast.

---

## How to Play

| Action                 | Input                                         |
| ---------------------- | --------------------------------------------- |
| Start the game         | Click **START**                               |
| Direct the fuel truck  | Left-click anywhere on the apron              |
| Upgrade the truck      | Click the **UPGRADE** button (or press **U**) |
| Restart after win/lose | Click **RESTART**                             |

Clicks on the HUD bar and buttons do **not** redirect the truck.

### Objective

1. Aircraft taxi in on the horizontal runways (top & bottom) and the vertical runways (left & right).
2. Click on the apron to drive the fuel truck toward a parked aircraft.
3. Get within range — fueling starts automatically and a progress bar fills above the plane.
4. Once full, the plane departs and you earn money based on its color.
5. Reach **50 planes fueled** to win.

### Runways open progressively

- **Runways 1 & 2** (horizontal) are open from the start.
- **Runway 3** (left, vertical) opens after **10 planes** are fueled.
- **Runway 4** (right, vertical) opens after **25 planes** are fueled.

When a runway opens, its center-line dashes turn **yellow** and its parking-stand rectangle turns **white**.

As runways open, payouts also scale up to keep the game profitable against the rising plane volume.

### Plane values

Base payout is scaled by a multiplier (+5% at 3rd runway, +10.25% at 4th), rounded up to the nearest $5, then a flat per-plane bonus is added (+$20 at 3rd, +$30 at 4th).

| Color  | 1–2 runways | 3rd open | 4th open |
| ------ | ----------: | -------: | -------: |
| Azure  |         $60 |      $85 |     $100 |
| Blue   |         $90 |     $115 |     $130 |
| Yellow |        $120 |     $150 |     $165 |
| Orange |        $150 |     $180 |     $200 |
| Red    |        $175 |     $205 |     $225 |

### Truck upgrades (8 levels, $500 → $4000)

Each upgrade:

- Increases truck **movement speed** (130 → 395 px/s)
- Decreases **fueling time** (2.5 → 0.6 s per plane)
- Changes the truck's **color and size** (gray → yellow → orange → red → purple → green → cyan → pink → gold)
- Preserves the truck's current destination and active fueling — a moving or fueling truck keeps doing its job through the upgrade

### Win / Lose

- **Win** — Fuel 50 planes. Fireworks light up the sky.
- **Lose** — A taxiing aircraft strikes a parked plane that wasn't fueled in time (the truck is destroyed in the explosion). Click **RESTART** to try again.

---

## Phase 1 — MVP (Complete)

- [x] Bevy project scaffolded, window opens
- [x] Top-down apron scene with runway strip
- [x] Aircraft taxis in from runway to parking stand
- [x] Fuel truck controlled by mouse click (point-and-click movement)
- [x] Collision detection — truck struck by taxiing aircraft triggers lose condition
- [x] Fueling mechanic — truck docks with parked aircraft, progress bar fills
- [x] Win screen on successful fueling
- [x] Crash screen on collision
- [x] R key restarts the round

---

## Phase 2 — Gameplay Loop (Complete)

- [x] START / RESTART buttons
- [x] HUD bar with money, current truck, and upgrade button
- [x] Per-runway fuel progress bars
- [x] Multiple plane colors with different payouts
- [x] Apron scenery — ATC tower, terminal building, three barrel-vault hangars

---

## Phase 3 — Continuous Mode (Complete)

- [x] Continuous play to **50 planes** (no discrete levels)
- [x] Four runways with progressive activation at 10 and 25 fueled
- [x] Spawn interval tightens as the player progresses
- [x] Truck upgrades (8 tiers) — faster movement + faster fueling, state-preserving (destination + fueling carry through)
- [x] Tiered payout bonuses (multiplier + flat) that scale as more runways open
- [x] Partial fueling persists on the aircraft when the truck is redirected
- [x] Crashes between taxiing and parked aircraft (with the truck caught in between)
- [x] Win screen with continuous fireworks
- [x] Visual polish — windshield + side windows, vertical "FUEL" label painted on the tank (rotates with the truck), contrast-tuned label color (red/yellow), pointer cursor on UI hover, distinct orange upgrade button, "COMING SOON" markers on inactive runways
- [x] Detailed hangars (corrugated arch, sliding door panels, red trim, lit windows) and trees scattered through the grass areas
- [x] UI clicks no longer redirect the truck

---

## Phase 4 — Refactor & Performance (Complete)

**Ground rule: no change in Phase 4 altered gameplay, balance, visuals, or behavior.** Phase 4 was a code-quality and organization pass only.

- [x] Extracted all components, marker types, and `GameState` into `src/components.rs`
- [x] Introduced query type aliases (`TruckIconFuelFilter`, `UpgTruckFuelFilter`, `WorldTruckFuelFilter`, `TruckLabelFilter`) to satisfy clippy `type_complexity`
- [x] Collapsed duplicated `if`/`else` branches surfaced by clippy `if_same_then_else`
- [x] `cargo check` clean (zero warnings) after the split

---

## Tech Stack

|             |                                          |
| ----------- | ---------------------------------------- |
| Language    | Rust 1.96                                |
| Game Engine | Bevy 0.15                                |
| Platform    | Windows (tested), Linux/macOS (untested) |
