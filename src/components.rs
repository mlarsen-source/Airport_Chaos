use bevy::prelude::*;
use std::collections::VecDeque;

// ── Components ─────────────────────────────────────────────────────────────
#[derive(Component)]
pub struct Aircraft {
    pub waypoints: VecDeque<(Vec2, f32)>,
    pub seg_start: Vec2,
    pub seg_start_scale: f32,
    pub speed: f32,
    pub parked: bool,
    pub departing: bool,
    pub runway: u32,
    pub base_value: f32,
    pub park_age: f32,
    pub will_collide: bool,
    pub fuel_progress: f32, // 0.0–1.0; persists when truck leaves mid-fuel
}

#[derive(Component)]
pub struct FuelTruck {
    pub destination: Option<Vec2>,
    pub speed: f32,
    pub fueling: bool,
}

#[derive(Component)] pub struct CrashText;
#[derive(Component)] pub struct LevelCompleteText;
#[derive(Component)] pub struct ProceedButton;
#[derive(Component)] pub struct RestartButton;
#[derive(Component)] pub struct RestartButtonText;
#[derive(Component)] pub struct TimerText;
#[derive(Component)] pub struct MoneyText;
#[derive(Component)] pub struct TruckUpgradeText;
#[derive(Component)] pub struct TruckUpgradeButton;
#[derive(Component)] pub struct TruckUpgradeBtnText;
#[derive(Component)] pub struct TruckIconBody;   // current-level truck body in bar
#[derive(Component)] pub struct TruckIconCab;    // current-level truck cab in bar
#[derive(Component)] pub struct TruckIconFuel;   // "FUEL" label on the current truck icon
#[derive(Component)] pub struct UpgTruckFuel;    // "FUEL" label on the upgrade preview truck icon
#[derive(Component)] pub struct WorldTruckFuel;  // "FUEL" label on the in-game truck
#[derive(Component)] pub struct ComingSoonLabel(pub u32); // runway index (2 or 3); hidden when that runway opens
#[derive(Component)] pub struct VerticalRunwayDash(pub u32); // runway index (2 or 3)
#[derive(Component)] pub struct ParkingStandSide(pub u32);   // runway index (2 or 3)
#[derive(Component)] pub struct UpgTruckBody;    // next-level truck body in upgrade button
#[derive(Component)] pub struct UpgTruckCab;     // next-level truck cab in upgrade button
#[derive(Component)] pub struct PlaneCountText;

#[derive(Component)]
pub struct RunwayFuelBar { pub runway: u32, pub is_fill: bool }

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub r: f32, pub g: f32, pub b: f32,
    pub scale_start: f32,
    pub scale_end: f32,
}

// ── Game state ─────────────────────────────────────────────────────────────
#[derive(Resource)]
pub struct GameState {
    pub crashed: bool,
    pub money: f32,
    pub truck_level: u32,
    pub last_payout: f32,
    pub planes_fueled: u32,
    pub payout_flash: f32,
    pub spawn_timers: [f32; 4],
    pub active_runways: usize,   // 2 → 3 → 4 based on planes_fueled
    pub crash_pos: Vec2,
    pub crash_anim_spawned: bool,
    pub rng_state: u32,
    pub started: bool,
    pub game_complete: bool,
    pub firework_timer: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            crashed: false,
            money: 0.0, truck_level: 0,
            last_payout: 0.0, planes_fueled: 0,
            payout_flash: 0.0,
            spawn_timers: [0.0, 7.0, 0.0, 0.0],
            active_runways: 2,
            crash_pos: Vec2::ZERO,
            crash_anim_spawned: false,
            rng_state: 0x9E3779B9,
            started: false,
            game_complete: false,
            firework_timer: 0.0,
        }
    }
}

impl GameState {
    pub fn next_rand(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.rng_state >> 16) as f32 / 65535.0  // 0.0 ..= 1.0
    }
}
