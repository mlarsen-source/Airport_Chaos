# Airport Ramp Chaos

A top-down airport ramp management game built in [Bevy](https://bevyengine.org/) (Rust).

You are a ramp supervisor directing ground service vehicles on a busy airport apron. Aircraft arrive on schedule and every vehicle has to do its job before departure — without getting in the way of a moving plane.

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

| Action | Input |
|---|---|
| Direct the fuel truck | Left-click anywhere on the apron |
| Restart after win/lose | Press **R** |

### Objective
1. Wait for the aircraft to taxi in from the runway and park (it turns **blue** when stopped).
2. Click the fuel truck to drive it toward the aircraft.
3. Get within range of the parked aircraft — the truck turns **green** and fueling begins.
4. The fuel progress bar fills over ~4 seconds. Don't get hit by the plane while it's taxiing!

### Win / Lose
- **Win** — Fueling completes before anything goes wrong.
- **Lose** — Fuel truck is struck by the taxiing aircraft (truck turns red, collision screen appears).

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

## Phase 2 — Planned

- [ ] Countdown timer — fuel the aircraft before departure time or lose
- [ ] HUD with timer display and round number
- [ ] Visual improvements — taxiway markings, parking stand lines, vehicle labels
- [ ] Aircraft rotation during taxi to reflect heading

---

## Phase 3 — Planned

- [ ] Multiple aircraft arriving in sequence
- [ ] Multiple service vehicles — baggage cart, catering truck
- [ ] Each aircraft requires all services before it can depart
- [ ] Player must coordinate multiple vehicles simultaneously

---

## Phase 4 — Planned

- [ ] Pathfinding with taxiway constraints — vehicles follow painted routes
- [ ] Vehicle-to-vehicle collision avoidance
- [ ] Progressive difficulty — tighter schedules, more aircraft, faster pace
- [ ] Scoring system based on on-time departures and accidents avoided

---

## Tech Stack

| | |
|---|---|
| Language | Rust 1.96 |
| Game Engine | Bevy 0.15 |
| Platform | Windows (tested), Linux/macOS (untested) |
