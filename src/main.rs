use bevy::prelude::*;
use bevy::winit::cursor::CursorIcon;
use bevy::window::SystemCursorIcon;
use bevy::window::PrimaryWindow;
use std::collections::VecDeque;

// ── Layout ─────────────────────────────────────────────────────────────────
const VERT_X: f32 = 360.0;
const VERT_W: f32 = 80.0;
const HORIZ_Y: f32 = 230.0;
const HORIZ_H: f32 = 80.0;
const APRON_X: f32 = VERT_X - VERT_W / 2.0;
const APRON_Y: f32 = HORIZ_Y - HORIZ_H / 2.0;
const TRUCK_BOUND_X: f32 = APRON_X - 5.0;
const TRUCK_BOUND_Y: f32 = APRON_Y - 5.0;

const RUNWAY_STANDS: [Vec2; 4] = [
    Vec2::new(0.0,     HORIZ_Y),
    Vec2::new(0.0,    -HORIZ_Y),
    Vec2::new(-VERT_X,  0.0),
    Vec2::new( VERT_X,  0.0),
];

const TRUCK_UPGRADE_COSTS: [f32; 5]  = [500.0, 1000.0, 1500.0, 2000.0, 2500.0];
const FUEL_DURATIONS: [f32; 6]       = [4.0, 3.5, 3.0, 2.5, 2.0, 1.5];
const TRUCK_SPEEDS:    [f32; 6]      = [130.0, 155.0, 180.0, 210.0, 240.0, 275.0];
const FUEL_RADIUS: f32 = 72.0;
const TRUCK_START: Vec2 = Vec2::ZERO;
const SPAWN_JITTER: f32 = 4.0;     // random ±2 s around the spawn interval
const WIN_PLANES: u32 = 50;
const RUNWAY3_OPEN_AT: u32 = 10;   // 3rd runway opens after this many planes fueled
const RUNWAY4_OPEN_AT: u32 = 20;   // 4th runway opens after this many planes fueled
const BAR_WIDTH: f32 = 60.0;
const BAR_Y0: f32 = HORIZ_Y + HORIZ_H / 2.0 + 15.0;
const BAR_Y1: f32 = -(HORIZ_Y + HORIZ_H / 2.0 + 15.0);

// ── Components ─────────────────────────────────────────────────────────────
#[derive(Component)]
struct Aircraft {
    waypoints: VecDeque<(Vec2, f32)>,
    seg_start: Vec2,
    seg_start_scale: f32,
    speed: f32,
    parked: bool,
    departing: bool,
    runway: u32,
    base_value: f32,
    park_age: f32,
    will_collide: bool,
    fuel_progress: f32, // 0.0–1.0; persists when truck leaves mid-fuel
}

#[derive(Component)]
struct FuelTruck {
    destination: Option<Vec2>,
    speed: f32,
    fueling: bool,
}

#[derive(Component)] struct CrashText;
#[derive(Component)] struct LevelCompleteText;
#[derive(Component)] struct ProceedButton;
#[derive(Component)] struct RestartButton;
#[derive(Component)] struct RestartButtonText;
#[derive(Component)] struct TimerText;
#[derive(Component)] struct MoneyText;
#[derive(Component)] struct TruckUpgradeText;
#[derive(Component)] struct TruckUpgradeButton;
#[derive(Component)] struct TruckUpgradeBtnText;
#[derive(Component)] struct TruckIconBody;   // current-level truck body in bar
#[derive(Component)] struct TruckIconCab;    // current-level truck cab in bar
#[derive(Component)] struct TruckIconFuel;   // "FUEL" label on the current truck icon
#[derive(Component)] struct UpgTruckFuel;    // "FUEL" label on the upgrade preview truck icon
#[derive(Component)] struct WorldTruckFuel;  // "FUEL" label on the in-game truck
#[derive(Component)] struct ComingSoonLabel(u32); // runway index (2 or 3); hidden when that runway opens
#[derive(Component)] struct VerticalRunwayDash(u32); // runway index (2 or 3)
#[derive(Component)] struct ParkingStandSide(u32);   // runway index (2 or 3)
#[derive(Component)] struct UpgTruckBody;    // next-level truck body in upgrade button
#[derive(Component)] struct UpgTruckCab;     // next-level truck cab in upgrade button
#[derive(Component)] struct PlaneCountText;

#[derive(Component)]
struct RunwayFuelBar { runway: u32, is_fill: bool }

#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime: f32,
    max_lifetime: f32,
    r: f32, g: f32, b: f32,
    scale_start: f32,
    scale_end: f32,
}

// ── Game state ─────────────────────────────────────────────────────────────
#[derive(Resource)]
struct GameState {
    crashed: bool,
    money: f32,
    truck_level: u32,
    last_payout: f32,
    planes_fueled: u32,
    payout_flash: f32,
    spawn_timers: [f32; 4],
    active_runways: usize,   // 2 → 3 → 4 based on planes_fueled
    crash_pos: Vec2,
    crash_anim_spawned: bool,
    rng_state: u32,
    started: bool,
    game_complete: bool,
    firework_timer: f32,
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
    fn next_rand(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.rng_state >> 16) as f32 / 65535.0  // 0.0 ..= 1.0
    }
}

// ── Approach / departure waypoints ─────────────────────────────────────────
fn runway_approach(runway: u32) -> (Vec2, f32, Vec<(Vec2, f32)>) {
    let (hi, mid, low) = (0.35_f32, 0.72_f32, 1.0_f32);
    match runway {
        0 => (Vec2::new(580.0, 310.0), hi,
              vec![(Vec2::new(420.0, HORIZ_Y), mid), (Vec2::new(0.0, HORIZ_Y), low)]),
        1 => (Vec2::new(-580.0, -310.0), hi,
              vec![(Vec2::new(-420.0, -HORIZ_Y), mid), (Vec2::new(0.0, -HORIZ_Y), low)]),
        2 => (Vec2::new(-VERT_X, 360.0), hi,
              vec![(Vec2::new(-VERT_X, HORIZ_Y + HORIZ_H * 0.6), mid), (Vec2::new(-VERT_X, 0.0), low)]),
        _ => (Vec2::new(VERT_X, -360.0), hi,
              vec![(Vec2::new(VERT_X, -(HORIZ_Y + HORIZ_H * 0.6)), mid), (Vec2::new(VERT_X, 0.0), low)]),
    }
}

fn runway_departure(runway: u32) -> Vec<(Vec2, f32)> {
    let (mid, hi) = (0.72_f32, 0.35_f32);
    match runway {
        0 => vec![(Vec2::new(-420.0, HORIZ_Y), mid), (Vec2::new(-580.0, 310.0), hi)],
        1 => vec![(Vec2::new(420.0, -HORIZ_Y), mid),  (Vec2::new(580.0, -310.0), hi)],
        2 => vec![(Vec2::new(-VERT_X, -(HORIZ_Y + HORIZ_H * 0.6)), mid), (Vec2::new(-VERT_X, -360.0), hi)],
        _ => vec![(Vec2::new(VERT_X, HORIZ_Y + HORIZ_H * 0.6), mid), (Vec2::new(VERT_X, 360.0), hi)],
    }
}

// Tank + cab colors for the CURRENT level
fn truck_level_colors(level: u32) -> (Color, Color) {
    match level {
        0 => (Color::srgb(0.72, 0.74, 0.76), Color::srgb(0.95, 0.76, 0.05)),
        1 => (Color::srgb(0.96, 0.88, 0.18), Color::srgb(0.90, 0.70, 0.05)),
        2 => (Color::srgb(0.97, 0.62, 0.08), Color::srgb(0.85, 0.42, 0.04)),
        3 => (Color::srgb(0.92, 0.30, 0.08), Color::srgb(0.72, 0.18, 0.04)),
        4 => (Color::srgb(0.65, 0.12, 0.80), Color::srgb(0.46, 0.06, 0.58)),
        _ => (Color::srgb(0.08, 0.88, 0.32), Color::srgb(0.04, 0.58, 0.20)),
    }
}

// Contrast color for the FUEL label on each truck level (red or yellow only)
fn truck_label_color(level: u32) -> Color {
    const RED:    Color = Color::srgb(0.95, 0.10, 0.10);
    const YELLOW: Color = Color::srgb(1.00, 0.92, 0.10);
    match level {
        0 => RED,    // red on gray
        1 => RED,    // red on yellow
        2 => RED,    // red on orange
        3 => YELLOW, // yellow on red
        4 => YELLOW, // yellow on purple
        _ => YELLOW, // yellow on green / other
    }
}

// Seconds between plane arrivals per runway, based on total planes fueled.
// Tuned so the game stays winnable as the player upgrades their truck.
fn spawn_interval_for_progress(fueled: u32) -> f32 {
    if      fueled <  RUNWAY3_OPEN_AT { 18.0 }   // 0–9, 2 runways
    else if fueled <  RUNWAY4_OPEN_AT { 15.0 }   // 10–19, 3 runways
    else if fueled <  40              { 13.0 }   // 20–39, 4 runways
    else if fueled <  60              { 11.0 }
    else if fueled <  80              { 10.0 }
    else                              {  9.0 }
}

// How many runways are active for the current plane count
fn active_runways_for_progress(fueled: u32) -> usize {
    if      fueled >= RUNWAY4_OPEN_AT { 4 }
    else if fueled >= RUNWAY3_OPEN_AT { 3 }
    else                              { 2 }
}

// Tank + cab colors for the NEXT upgrade level (shown as icon preview)
fn truck_next_level_colors(current: u32) -> (Color, Color) {
    match current + 1 {
        1 => (Color::srgb(0.96, 0.88, 0.18), Color::srgb(0.90, 0.70, 0.05)),
        2 => (Color::srgb(0.97, 0.62, 0.08), Color::srgb(0.85, 0.42, 0.04)),
        3 => (Color::srgb(0.92, 0.30, 0.08), Color::srgb(0.72, 0.18, 0.04)),
        4 => (Color::srgb(0.65, 0.12, 0.80), Color::srgb(0.46, 0.06, 0.58)),
        5 => (Color::srgb(0.08, 0.88, 0.32), Color::srgb(0.04, 0.58, 0.20)),
        _ => (Color::srgb(0.28, 0.28, 0.30), Color::srgb(0.22, 0.22, 0.24)), // max
    }
}

// Color → base payout. Time bonus applied in check_fueling.
// Label shown in HUD payout flash.
fn plane_color_info(idx: u8) -> (Color, f32, &'static str) {
    match idx % 5 {
        0 => (Color::srgb(0.22, 0.62, 0.88),  50.0, "Azure"),
        1 => (Color::srgb(0.30, 0.65, 1.00),  75.0, "Blue"),
        2 => (Color::srgb(1.00, 0.90, 0.10), 100.0, "Yellow"),
        3 => (Color::srgb(1.00, 0.52, 0.05), 125.0, "Orange"),
        _ => (Color::srgb(0.90, 0.15, 0.15), 150.0, "Red"),
    }
}

// ── Main ───────────────────────────────────────────────────────────────────
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Airport Ramp Chaos".into(),
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            spawn_planes,
            taxi_aircraft,
            handle_click,
            move_fuel_truck,
            check_fueling,
            update_hud,
            update_truck_label,
            handle_upgrades,
            handle_restart,
            spawn_crash_effects,
            update_particles,
        ))
        .add_systems(Update, (
            update_active_runways,
            spawn_fireworks,
            update_cursor_icon,
            update_fuel_labels,
            update_world_fuel_label,
        ))
        .run();
}

// ── Setup ──────────────────────────────────────────────────────────────────
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Grass background
    commands.spawn((
        Sprite { color: Color::srgb(0.22, 0.52, 0.18), custom_size: Some(Vec2::new(1200.0, 900.0)), ..default() },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Center apron
    commands.spawn((
        Sprite { color: Color::srgb(0.30, 0.31, 0.30), custom_size: Some(Vec2::new(APRON_X * 2.0, APRON_Y * 2.0)), ..default() },
        Transform::from_xyz(0.0, 0.0, 0.1),
    ));

    // Horizontal runways (active: full color)
    for sign in [1.0_f32, -1.0] {
        commands.spawn((
            Sprite { color: Color::srgb(0.18, 0.18, 0.19), custom_size: Some(Vec2::new(1200.0, HORIZ_H)), ..default() },
            Transform::from_xyz(0.0, sign * HORIZ_Y, 0.2),
        ));
        // Yellow centerline dashes
        for seg in -6i32..=6 {
            commands.spawn((
                Sprite { color: Color::srgb(0.98, 0.88, 0.08), custom_size: Some(Vec2::new(45.0, 4.0)), ..default() },
                Transform::from_xyz(seg as f32 * 80.0, sign * HORIZ_Y, 0.3),
            ));
        }
    }

    // Vertical runways (dimmed — not yet active)
    for sign in [1.0_f32, -1.0] {
        let rwy: u32 = if sign > 0.0 { 3 } else { 2 };
        commands.spawn((
            Sprite { color: Color::srgb(0.13, 0.13, 0.14), custom_size: Some(Vec2::new(VERT_W, 900.0)), ..default() },
            Transform::from_xyz(sign * VERT_X, 0.0, 0.2),
        ));
        for seg in -5i32..=5 {
            commands.spawn((
                VerticalRunwayDash(rwy),
                Sprite { color: Color::srgb(0.35, 0.35, 0.36), custom_size: Some(Vec2::new(4.0, 45.0)), ..default() },
                Transform::from_xyz(sign * VERT_X, seg as f32 * 80.0, 0.3),
            ));
        }
        // "COMING SOON" label — hidden when this runway opens.
        // sign = +1 → right side (runway 3), sign = -1 → left side (runway 2)
        let label_runway = if sign > 0.0 { 3 } else { 2 };
        commands.spawn((
            ComingSoonLabel(label_runway),
            Text2d::new("COMING\nSOON"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgba(0.55, 0.55, 0.55, 0.9)),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_xyz(sign * VERT_X, 0.0, 5.0),
            Visibility::Visible,
        ));
    }

    // Parking stand markers — runways 0 & 1 bright, 2 & 3 dim (recolored on open)
    for i in 0..4usize {
        let s = RUNWAY_STANDS[i];
        let c = if i < 2 { Color::srgb(0.90, 0.90, 0.90) } else { Color::srgb(0.30, 0.30, 0.30) };
        let (bw, bh): (f32, f32) = if i < 2 { (100.0, 36.0) } else { (36.0, 100.0) };
        let t = 3.0;
        let spawn_side = |commands: &mut Commands, size: Vec2, x: f32, y: f32| {
            let mut e = commands.spawn((
                Sprite { color: c, custom_size: Some(size), ..default() },
                Transform::from_xyz(x, y, 0.4),
            ));
            if i >= 2 { e.insert(ParkingStandSide(i as u32)); }
        };
        spawn_side(&mut commands, Vec2::new(bw, t), s.x, s.y + bh / 2.0);
        spawn_side(&mut commands, Vec2::new(bw, t), s.x, s.y - bh / 2.0);
        spawn_side(&mut commands, Vec2::new(t, bh), s.x - bw / 2.0, s.y);
        spawn_side(&mut commands, Vec2::new(t, bh), s.x + bw / 2.0, s.y);
    }

    // ── Airport structures ──────────────────────────────────────────────────
    let s_hx = APRON_X - 46.0;   // 274
    let s_hy = APRON_Y - 42.0;   // 148

    // ── Hangars: front-profile barrel-vault view ─────────────────────────────
    // Shows the recognisable arched silhouette + giant door opening.
    let h_metal  = Color::srgb(0.62, 0.63, 0.67);
    let h_shadow = Color::srgb(0.40, 0.41, 0.44);
    let h_door   = Color::srgb(0.07, 0.07, 0.08);
    let h_base   = Color::srgb(0.33, 0.34, 0.36);

    for (cx, cy) in [(-s_hx, s_hy), (s_hx, s_hy), (-s_hx, -s_hy)] {
        // Concrete base sill
        commands.spawn((Sprite { color: h_base,  custom_size: Some(Vec2::new(92.0, 8.0)),  ..default() }, Transform::from_xyz(cx, cy - 32.0, 1.00)));
        // Side walls flanking the door opening
        commands.spawn((Sprite { color: h_shadow, custom_size: Some(Vec2::new(9.0, 28.0)), ..default() }, Transform::from_xyz(cx - 41.5, cy - 18.0, 1.01)));
        commands.spawn((Sprite { color: h_shadow, custom_size: Some(Vec2::new(9.0, 28.0)), ..default() }, Transform::from_xyz(cx + 41.5, cy - 18.0, 1.01)));
        // Large door void — nearly full width
        commands.spawn((Sprite { color: h_door,  custom_size: Some(Vec2::new(74.0, 28.0)), ..default() }, Transform::from_xyz(cx, cy - 18.0, 1.01)));
        // Barrel-vault arch: 6 stacked rects narrow to the apex
        for (i, &w) in [90.0_f32, 82.0, 72.0, 58.0, 40.0, 22.0].iter().enumerate() {
            commands.spawn((Sprite { color: h_metal, custom_size: Some(Vec2::new(w, 5.0)), ..default() }, Transform::from_xyz(cx, cy - 4.0 + i as f32 * 5.0, 1.02 + i as f32 * 0.01)));
        }
    }

    // ── ATC Tower (bottom-right): slender shaft, wide glass cab, control building
    let (tx, ty) = (s_hx, -s_hy);   // 274, -148

    // Foundation slab (clean, simple)
    commands.spawn((Sprite { color: Color::srgb(0.28, 0.29, 0.32), custom_size: Some(Vec2::new(38.0, 10.0)), ..default() }, Transform::from_xyz(tx,        ty - 36.0, 1.00)));
    // Adjacent control / equipment building — base aligned with tower foundation
    commands.spawn((Sprite { color: Color::srgb(0.40, 0.41, 0.44), custom_size: Some(Vec2::new(36.0, 22.0)), ..default() }, Transform::from_xyz(tx - 28.0, ty - 30.0, 1.00)));
    // Red roof strip at the top of the building
    commands.spawn((Sprite { color: Color::srgb(0.82, 0.12, 0.10), custom_size: Some(Vec2::new(36.0,  5.0)), ..default() }, Transform::from_xyz(tx - 28.0, ty - 21.5, 1.01)));
    commands.spawn((Sprite { color: Color::srgb(0.22, 0.52, 0.70), custom_size: Some(Vec2::new(7.0,   9.0)), ..default() }, Transform::from_xyz(tx - 35.0, ty - 30.0, 1.01))); // window
    commands.spawn((Sprite { color: Color::srgb(0.22, 0.52, 0.70), custom_size: Some(Vec2::new(7.0,   9.0)), ..default() }, Transform::from_xyz(tx - 22.0, ty - 30.0, 1.01))); // window
    // Wide concrete shaft — 14 px so it reads as a proper tower column
    commands.spawn((Sprite { color: Color::srgb(0.46, 0.47, 0.50), custom_size: Some(Vec2::new(14.0, 82.0)), ..default() }, Transform::from_xyz(tx,        ty,        1.01)));
    // Glass control cab — 44 px wide (7× the shaft), dramatically overhanging
    commands.spawn((Sprite { color: Color::srgb(0.20, 0.60, 0.82), custom_size: Some(Vec2::new(44.0, 19.0)), ..default() }, Transform::from_xyz(tx,        ty + 29.0, 1.10)));
    // Cab structural floor / bottom rail
    commands.spawn((Sprite { color: Color::srgb(0.13, 0.40, 0.58), custom_size: Some(Vec2::new(44.0,  5.0)), ..default() }, Transform::from_xyz(tx,        ty + 19.5, 1.11)));
    // Walkway railing strips (top and bottom, wider than cab)
    let rail = Color::srgb(0.50, 0.52, 0.55);
    commands.spawn((Sprite { color: rail, custom_size: Some(Vec2::new(48.0, 2.5)), ..default() }, Transform::from_xyz(tx, ty + 39.5, 1.12)));
    commands.spawn((Sprite { color: rail, custom_size: Some(Vec2::new(48.0, 2.5)), ..default() }, Transform::from_xyz(tx, ty + 18.0, 1.12)));
    // Window-frame columns (3 dividers = 4 glass bays)
    let wc = Color::srgb(0.12, 0.15, 0.19);
    for dx in [-14.0_f32, 0.0, 14.0] {
        commands.spawn((Sprite { color: wc, custom_size: Some(Vec2::new(2.0, 19.0)), ..default() }, Transform::from_xyz(tx + dx, ty + 29.0, 1.13)));
    }
    // Antenna mast
    commands.spawn((Sprite { color: Color::srgb(0.62, 0.62, 0.65), custom_size: Some(Vec2::new(2.0, 22.0)), ..default() }, Transform::from_xyz(tx, ty + 53.5, 1.20)));
    // Red aviation obstruction light
    commands.spawn((Sprite { color: Color::srgb(0.95, 0.10, 0.06), custom_size: Some(Vec2::new(6.0,  6.0)),  ..default() }, Transform::from_xyz(tx, ty + 65.5, 1.21)));

    spawn_truck(&mut commands, 0, TRUCK_START);

    // Truck label
    commands.spawn((
        PlaneCountText, // reuse as truck label carrier — actually use a separate entity
        Text2d::new("FUEL"),
        TextFont { font_size: 7.5, ..default() },
        TextColor(truck_label_color(0)),
        Transform::from_xyz(TRUCK_START.x, TRUCK_START.y + 20.0, 5.0),
    ));

    // ── UI ─────────────────────────────────────────────────────────────────
    let center = Node {
        position_type: PositionType::Absolute,
        top: Val::Percent(30.0),
        left: Val::Percent(20.0),
        right: Val::Percent(20.0),
        ..default()
    };

    // Crash overlay
    commands.spawn((CrashText,
        Text::new("GAME OVER"),
        TextFont { font_size: 88.0, ..default() },
        TextColor(Color::srgba(1.0, 0.12, 0.12, 0.0)),
        TextLayout::new_with_justify(JustifyText::Center),
        center.clone()));

    // Level complete overlay
    commands.spawn((LevelCompleteText,
        Text::new(""),
        TextFont { font_size: 48.0, ..default() },
        TextColor(Color::srgba(0.1, 1.0, 0.35, 0.0)),
        TextLayout::new_with_justify(JustifyText::Center),
        center));

    // Restart / Start button — visible from the start with "START" label
    commands.spawn((
        RestartButton, Button,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(55.0), left: Val::Px(422.0),
            width: Val::Px(180.0), height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.35, 0.9)),
        BorderRadius::all(Val::Px(25.0)),
        Visibility::Visible,
    )).with_children(|p| {
        p.spawn((RestartButtonText, Text::new("START"), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE)));
    });

    // Proceed button
    commands.spawn((
        ProceedButton, Button,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(62.0), left: Val::Px(422.0),
            width: Val::Px(180.0), height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.65, 0.28, 0.0)),
        BorderRadius::all(Val::Px(25.0)),
        Visibility::Hidden,
    )).with_children(|p| {
        p.spawn((Text::new("PROCEED"), TextFont { font_size: 28.0, ..default() }, TextColor(Color::WHITE)));
    });

    // ── Top HUD bar — sits between the two vertical runways (world x ±320 = screen x 192/832)
    let (cur_tank, cur_cab) = truck_level_colors(0);
    let (upg_tank, upg_cab) = truck_next_level_colors(0);
    let wheel_col = Color::srgb(0.10, 0.10, 0.12);
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(4.0), left: Val::Px(192.0), right: Val::Px(192.0),
            height: Val::Px(46.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceAround,
            padding: UiRect::horizontal(Val::Px(16.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.14, 0.17, 0.24, 0.97)),
        BorderRadius::all(Val::Px(8.0)),
    )).with_children(|bar| {
        // ── Earnings ──────────────────────────────────────────────────────
        bar.spawn((
            MoneyText,
            Text::new("$0.00"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(0.25, 0.95, 0.50)),
        ));

        // ── Current truck (side view) + level text ────────────────────────
        bar.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        }).with_children(|grp| {
            // Side-view truck icon: body (left) + cab (right) + 2 wheels
            grp.spawn(Node {
                width: Val::Px(55.0), height: Val::Px(30.0), ..default()
            }).with_children(|t| {
                // Tank body
                t.spawn((
                    TruckIconBody,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0), top: Val::Px(4.0),
                        width: Val::Px(34.0), height: Val::Px(18.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(cur_tank),
                    BorderRadius::all(Val::Px(2.0)),
                )).with_children(|b| {
                    b.spawn((TruckIconFuel, Text::new("FUEL"), TextFont { font_size: 9.0, ..default() }, TextColor(truck_label_color(0))));
                });
                // Cab
                t.spawn((
                    TruckIconCab,
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(0.0), top: Val::Px(0.0),
                        width: Val::Px(16.0), height: Val::Px(22.0),
                        ..default()
                    },
                    BackgroundColor(cur_cab),
                    BorderRadius::all(Val::Px(2.0)),
                )).with_children(|cab| {
                    // Side window
                    cab.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(2.0), top: Val::Px(3.0),
                            width: Val::Px(12.0), height: Val::Px(8.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.65, 0.85, 1.0, 0.95)),
                        BorderRadius::all(Val::Px(1.5)),
                    ));
                });
                // Rear wheel
                t.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(4.0), bottom: Val::Px(0.0), width: Val::Px(9.0), height: Val::Px(9.0), ..default() }, BackgroundColor(wheel_col), BorderRadius::all(Val::Px(4.5))));
                // Front wheel
                t.spawn((Node { position_type: PositionType::Absolute, right: Val::Px(4.0), bottom: Val::Px(0.0), width: Val::Px(9.0), height: Val::Px(9.0), ..default() }, BackgroundColor(wheel_col), BorderRadius::all(Val::Px(4.5))));
            });
            // Level text
            grp.spawn((
                TruckUpgradeText,
                Text::new("LV 0"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.85, 0.86, 0.90)),
            ));
        });

        // ── UPGRADE label + button (grouped so they stay together) ───────
        bar.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        }).with_children(|grp| {
            grp.spawn((
                Text::new("UPGRADE"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.95, 0.85, 0.30)),
            ));

            // ── Upgrade button: next-level truck + cost ───────────────────
            grp.spawn((
                TruckUpgradeButton, Button,
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.55, 0.10)),
                BorderColor(Color::srgb(1.0, 0.85, 0.25)),
                BorderRadius::all(Val::Px(8.0)),
            )).with_children(|btn| {
                // Next-level truck side view (slightly smaller)
                btn.spawn(Node {
                    width: Val::Px(46.0), height: Val::Px(26.0), ..default()
                }).with_children(|t| {
                    t.spawn((
                        UpgTruckBody,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0), top: Val::Px(3.0),
                            width: Val::Px(28.0), height: Val::Px(15.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(upg_tank),
                        BorderRadius::all(Val::Px(2.0)),
                    )).with_children(|b| {
                        b.spawn((UpgTruckFuel, Text::new("FUEL"), TextFont { font_size: 8.0, ..default() }, TextColor(truck_label_color(1))));
                    });
                    t.spawn((
                        UpgTruckCab,
                        Node { position_type: PositionType::Absolute, right: Val::Px(0.0), top: Val::Px(0.0), width: Val::Px(14.0), height: Val::Px(18.0), ..default() },
                        BackgroundColor(upg_cab),
                        BorderRadius::all(Val::Px(2.0)),
                    )).with_children(|cab| {
                        cab.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(2.0), top: Val::Px(2.0),
                                width: Val::Px(10.0), height: Val::Px(7.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.65, 0.85, 1.0, 0.95)),
                            BorderRadius::all(Val::Px(1.0)),
                        ));
                    });
                    t.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(3.0), bottom: Val::Px(0.0), width: Val::Px(8.0), height: Val::Px(8.0), ..default() }, BackgroundColor(wheel_col), BorderRadius::all(Val::Px(4.0))));
                    t.spawn((Node { position_type: PositionType::Absolute, right: Val::Px(3.0), bottom: Val::Px(0.0), width: Val::Px(8.0), height: Val::Px(8.0), ..default() }, BackgroundColor(wheel_col), BorderRadius::all(Val::Px(4.0))));
                });
                // Cost
                btn.spawn((
                    TruckUpgradeBtnText,
                    Text::new("$500"),
                    TextFont { font_size: 15.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });

    // Payout / status flash (below the HUD bar)
    commands.spawn((TimerText,
        Text::new(""),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgb(0.9, 0.82, 0.1)),
        TextLayout::new_with_justify(JustifyText::Center),
        Node { position_type: PositionType::Absolute, top: Val::Px(58.0), left: Val::Px(0.0), right: Val::Px(0.0), ..default() }));

    // Per-runway fuel bars — horizontal runways 0&1 and vertical runways 2&3
    // Runways 2&3 bars sit just above their parking stands (at world x=±VERT_X, y=0)
    for (rwy, bar_x, bar_y) in [
        (0u32,    0.0,  BAR_Y0),
        (1u32,    0.0,  BAR_Y1),
        (2u32, -VERT_X, 22.0),
        (3u32,  VERT_X, 22.0),
    ] {
        commands.spawn((
            RunwayFuelBar { runway: rwy, is_fill: false },
            Sprite { color: Color::srgb(0.10, 0.10, 0.10), custom_size: Some(Vec2::new(BAR_WIDTH, 8.0)), ..default() },
            Transform::from_xyz(bar_x, bar_y, 3.5),
            Visibility::Hidden,
        ));
        commands.spawn((
            RunwayFuelBar { runway: rwy, is_fill: true },
            Sprite {
                color: Color::srgb(0.1, 0.85, 0.3),
                custom_size: Some(Vec2::new(0.01, 6.0)),
                anchor: bevy::sprite::Anchor::CenterLeft,
                ..default()
            },
            Transform::from_xyz(bar_x - BAR_WIDTH / 2.0, bar_y, 3.6),
            Visibility::Hidden,
        ));
    }
}

// ── Spawn helpers ──────────────────────────────────────────────────────────
fn spawn_aircraft(commands: &mut Commands, runway: u32, color_idx: u8, will_collide: bool) {
    let (start, start_scale, wps) = runway_approach(runway);
    let (body_color, base_value, _) = plane_color_info(color_idx);
    commands.spawn((
        Aircraft {
            waypoints: wps.into(),
            seg_start: start, seg_start_scale: start_scale,
            speed: 130.0, parked: false, departing: false, runway,
            base_value, park_age: 0.0, will_collide, fuel_progress: 0.0,
        },
        Sprite { color: body_color, custom_size: Some(Vec2::new(11.0, 68.0)), ..default() },
        Transform { translation: start.extend(2.0), scale: Vec3::splat(start_scale), ..default() },
    )).with_children(|p| {
        let eng = Color::srgb(0.16, 0.16, 0.18);
        // Wings — always white
        p.spawn((Sprite { color: Color::srgb(0.95, 0.95, 0.95), custom_size: Some(Vec2::new(62.0, 11.0)), ..default() }, Transform::from_xyz(0.0, 7.0, -0.1)));
        // Engine nacelles under each wing
        p.spawn((Sprite { color: eng, custom_size: Some(Vec2::new(9.0, 16.0)), ..default() }, Transform::from_xyz(-17.0, 3.0, -0.2)));
        p.spawn((Sprite { color: eng, custom_size: Some(Vec2::new(9.0, 16.0)), ..default() }, Transform::from_xyz( 17.0, 3.0, -0.2)));
        // Horizontal tail — always white
        p.spawn((Sprite { color: Color::srgb(0.95, 0.95, 0.95), custom_size: Some(Vec2::new(28.0, 8.0)), ..default() }, Transform::from_xyz(0.0, -26.0, -0.1)));
        // Vertical tail fin
        p.spawn((Sprite { color: body_color, custom_size: Some(Vec2::new(4.0, 16.0)), ..default() }, Transform::from_xyz(0.0, -22.0, -0.05)));
        // Cockpit windows — dark
        p.spawn((Sprite { color: Color::srgb(0.18, 0.22, 0.38), custom_size: Some(Vec2::new(7.0, 6.0)), ..default() }, Transform::from_xyz(0.0, 25.0, 0.1)));
    });
}

fn spawn_truck(commands: &mut Commands, level: u32, pos: Vec2) {
    let (tw, th, tc, cc) = match level {
        0 => (22.0_f32, 28.0_f32, Color::srgb(0.72, 0.74, 0.76), Color::srgb(0.95, 0.76, 0.05)),
        1 => (24.0, 33.0, Color::srgb(0.96, 0.88, 0.18), Color::srgb(0.90, 0.70, 0.05)),
        2 => (26.0, 38.0, Color::srgb(0.97, 0.62, 0.08), Color::srgb(0.85, 0.42, 0.04)),
        3 => (28.0, 43.0, Color::srgb(0.92, 0.30, 0.08), Color::srgb(0.72, 0.18, 0.04)),
        4 => (30.0, 48.0, Color::srgb(0.65, 0.12, 0.80), Color::srgb(0.46, 0.06, 0.58)),
        _ => (32.0, 54.0, Color::srgb(0.08, 0.88, 0.32), Color::srgb(0.04, 0.58, 0.20)),
    };
    let cab_h = 12.0_f32;
    let cab_y = th / 2.0 + cab_h / 2.0 + 1.0;
    commands.spawn((
        FuelTruck { destination: None, speed: TRUCK_SPEEDS[level.min(5) as usize], fueling: false },
        Sprite { color: tc, custom_size: Some(Vec2::new(tw, th)), ..default() },
        Transform::from_xyz(pos.x, pos.y, 2.0),
    )).with_children(|p| {
        // Cab (front of truck)
        p.spawn((Sprite { color: cc, custom_size: Some(Vec2::new(tw - 2.0, cab_h)), ..default() }, Transform::from_xyz(0.0, cab_y, 0.1)));
        // Windshield on the cab — light blue strip at the very front
        p.spawn((
            Sprite { color: Color::srgba(0.65, 0.85, 1.0, 0.95), custom_size: Some(Vec2::new(tw - 6.0, 3.5)), ..default() },
            Transform::from_xyz(0.0, cab_y + cab_h / 2.0 - 2.5, 0.2),
        ));
        // Rear bumper
        p.spawn((Sprite { color: Color::srgb(0.25, 0.25, 0.27), custom_size: Some(Vec2::new(5.0, 5.0)), ..default() }, Transform::from_xyz(0.0, -(th / 2.0 + 3.0), 0.1)));
        // Highlight band
        p.spawn((Sprite { color: Color::srgba(1.0, 1.0, 1.0, 0.25), custom_size: Some(Vec2::new(tw, 4.0)), ..default() }, Transform::from_xyz(0.0, 4.0, 0.05)));
    });

    // "FUEL" label — standalone (not parented) so it never rotates with the truck.
    // Vertical orientation, runs along the tank from front to back.
    commands.spawn((
        WorldTruckFuel,
        Text2d::new("F\nU\nE\nL"),
        TextFont { font_size: 13.0, ..default() },
        TextColor(truck_label_color(level)),
        TextLayout::new_with_justify(JustifyText::Center),
        Transform::from_xyz(pos.x, pos.y, 3.0),
    ));
}


// ── Systems ────────────────────────────────────────────────────────────────
fn spawn_planes(
    time: Res<Time>,
    mut gs: ResMut<GameState>,
    mut commands: Commands,
    ac_q: Query<&Aircraft>,
) {
    if gs.crashed || gs.game_complete || !gs.started { return; }

    let interval = spawn_interval_for_progress(gs.planes_fueled);

    let mut occupied      = [false; 4];
    let mut crash_inbound = [false; 4];
    for ac in &ac_q {
        let r = ac.runway as usize;
        if r < 4 {
            if ac.will_collide    { crash_inbound[r] = true; }
            else if !ac.departing { occupied[r]      = true; }
        }
    }

    for i in 0..gs.active_runways {
        gs.spawn_timers[i] -= time.delta_secs();
        if gs.spawn_timers[i] <= 0.0 {
            if crash_inbound[i] {
                gs.spawn_timers[i] = interval;
            } else if !occupied[i] {
                let color_idx = (gs.next_rand() * 5.0) as u8;
                spawn_aircraft(&mut commands, i as u32, color_idx, false);
                let jitter = gs.next_rand();
                gs.spawn_timers[i] = interval - SPAWN_JITTER / 2.0 + jitter * SPAWN_JITTER;
            } else {
                let crash_color = (gs.next_rand() * 5.0) as u8;
                spawn_aircraft(&mut commands, i as u32, crash_color, true);
                gs.spawn_timers[i] = interval;
            }
        }
    }
}

fn taxi_aircraft(
    time: Res<Time>,
    mut gs: ResMut<GameState>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Aircraft, &mut Transform, &mut Sprite)>,
) {
    if gs.crashed || gs.game_complete { return; }

    // Snapshot which runways actually have a plane parked and waiting for fuel.
    // Departing planes are NOT included — they no longer block the stand.
    let parked_runways: Vec<u32> = q.iter()
        .filter(|(_, ac, _, _)| ac.parked && !ac.departing && !ac.will_collide)
        .map(|(_, ac, _, _)| ac.runway)
        .collect();

    for (entity, mut ac, mut tf, _sp) in q.iter_mut() {
        let Some(&(target, target_scale)) = ac.waypoints.front() else {
            if ac.departing { commands.entity(entity).despawn_recursive(); }
            continue;
        };
        let diff = target - tf.translation.truncate();
        let dist = diff.length();
        let seg_len = ac.seg_start.distance(target);
        let t = if seg_len > 0.01 { 1.0 - (dist / seg_len).clamp(0.0, 1.0) } else { 1.0 };
        tf.scale = Vec3::splat(ac.seg_start_scale + (target_scale - ac.seg_start_scale) * t);

        if dist < 4.0 {
            tf.translation = target.extend(2.0);
            tf.scale = Vec3::splat(target_scale);
            ac.waypoints.pop_front();
            ac.seg_start = target; ac.seg_start_scale = target_scale;
            if ac.waypoints.is_empty() && !ac.parked && !ac.departing {
                if ac.will_collide {
                    if parked_runways.contains(&ac.runway) {
                        // Actual collision — a plane is still parked at this stand
                        gs.crashed = true;
                        gs.crash_pos = tf.translation.truncate();
                    } else {
                        // Player fueled in time — collision plane just parks normally
                        ac.parked = true;
                        ac.will_collide = false;
                    }
                } else {
                    ac.parked = true;
                }
            }
        } else {
            let dir = diff.normalize();
            tf.rotation = Quat::from_rotation_z(dir.to_angle() - std::f32::consts::FRAC_PI_2);
            tf.translation += (dir * ac.speed * time.delta_secs()).extend(0.0);
        }
        if ac.parked { ac.park_age += time.delta_secs(); }
    }
}

fn handle_click(
    mouse: Res<ButtonInput<MouseButton>>,
    gs: Res<GameState>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    ui_q: Query<&Interaction>,
    mut truck_q: Query<&mut FuelTruck>,
) {
    if gs.crashed || !gs.started || !mouse.just_pressed(MouseButton::Left) { return; }
    // Ignore clicks that land on any UI element (HUD bar, buttons).
    if ui_q.iter().any(|i| matches!(i, Interaction::Hovered | Interaction::Pressed)) { return; }
    let window = windows.single();
    let (cam, cam_tf) = camera_q.single();
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world) = cam.viewport_to_world_2d(cam_tf, cursor) else { return };

    let dest = Vec2::new(
        world.x.clamp(-TRUCK_BOUND_X, TRUCK_BOUND_X),
        world.y.clamp(-TRUCK_BOUND_Y, TRUCK_BOUND_Y),
    );
    for mut truck in &mut truck_q {
        truck.fueling     = false;       // interrupt any active fueling
        truck.destination = Some(dest);
    }
}

fn move_fuel_truck(
    time: Res<Time>,
    gs: Res<GameState>,
    mut q: Query<(&mut FuelTruck, &mut Transform)>,
) {
    if gs.crashed { return; }
    for (mut truck, mut tf) in &mut q {
        if truck.fueling { continue; }
        let Some(dest) = truck.destination else { continue };
        let diff = dest - tf.translation.truncate();
        if diff.length() < 3.0 {
            tf.translation = dest.extend(2.0);
            truck.destination = None;
        } else {
            let dir = diff.normalize();
            tf.rotation = Quat::from_rotation_z(dir.to_angle() - std::f32::consts::FRAC_PI_2);
            tf.translation += (dir * truck.speed * time.delta_secs()).extend(0.0);
        }
        tf.translation.x = tf.translation.x.clamp(-TRUCK_BOUND_X, TRUCK_BOUND_X);
        tf.translation.y = tf.translation.y.clamp(-TRUCK_BOUND_Y, TRUCK_BOUND_Y);
    }
}

fn check_fueling(
    time: Res<Time>,
    mut gs: ResMut<GameState>,
    mut ac_q:    Query<(Entity, &Transform, &mut Aircraft)>,
    mut truck_q: Query<(&Transform, &mut FuelTruck), (Without<Aircraft>, Without<RunwayFuelBar>)>,
    mut bar_q:   Query<(&RunwayFuelBar, &mut Sprite, &mut Visibility), (Without<Aircraft>, Without<FuelTruck>)>,
) {
    if gs.crashed { return; }
    let Ok((tr_tf, mut truck)) = truck_q.get_single_mut() else { return };
    let tr_pos = tr_tf.translation.truncate();

    // Find a parked plane within range — only fuel it if truck is stationary
    // (destination == None).  When destination is set the player has clicked
    // away; respect that even if the truck hasn't moved yet.
    let target: Option<(Entity, u32, f32)> = {
        let mut found = None;
        for (e, ac_tf, ac) in ac_q.iter() {
            if ac.parked && !ac.departing
                && ac_tf.translation.truncate().distance(tr_pos) < FUEL_RADIUS
            {
                found = Some((e, ac.runway, ac.base_value));
                break;
            }
        }
        found
    };

    match target {
        Some((entity, runway, base_value)) if truck.destination.is_none() => {
            // Stationary and in range — fuel this plane
            if !truck.fueling { truck.fueling = true; }

            let dur = FUEL_DURATIONS[gs.truck_level as usize];

            if let Ok((_, _, mut ac)) = ac_q.get_mut(entity) {
                ac.fuel_progress = (ac.fuel_progress + time.delta_secs() / dur).min(1.0);

                if ac.fuel_progress >= 1.0 {
                    // Done — send the aircraft on its way
                    for wp in runway_departure(runway) { ac.waypoints.push_back(wp); }
                    ac.parked    = false;
                    ac.departing = true;
                    ac.speed     = 310.0;

                    let bonus  = 1.0_f32;
                    let earned = (base_value * bonus).round();
                    gs.money        += earned;
                    gs.last_payout   = earned;
                    gs.planes_fueled += 1;
                    gs.payout_flash  = 2.5;

                    truck.fueling = false;
                    let new_active = active_runways_for_progress(gs.planes_fueled);
                    if new_active > gs.active_runways {
                        // A new runway just opened — stagger its first spawn
                        let interval = spawn_interval_for_progress(gs.planes_fueled);
                        for i in gs.active_runways..new_active {
                            gs.spawn_timers[i] = interval * 0.5;
                        }
                        gs.active_runways = new_active;
                    }
                    if gs.planes_fueled >= WIN_PLANES {
                        gs.game_complete = true;
                    }
                }
            }
        }
        _ => {
            // Out of range or player clicked away — stop the fueling flag but
            // leave fuel_progress on the aircraft so it resumes on return.
            truck.fueling = false;
        }
    }

    // Bars: visible + filled for any parked plane that has started fueling,
    // even while the truck is elsewhere.
    let parked_progress: Vec<(u32, f32)> = ac_q.iter()
        .filter(|(_, _, ac)| ac.parked && !ac.departing && ac.fuel_progress > 0.0)
        .map(|(_, _, ac)| (ac.runway, ac.fuel_progress))
        .collect();

    for (rwy_bar, mut bar_sp, mut bar_vis) in &mut bar_q {
        let prog = parked_progress.iter()
            .find(|(r, _)| *r == rwy_bar.runway)
            .map(|(_, p)| *p)
            .unwrap_or(0.0);

        if prog > 0.0 {
            *bar_vis = Visibility::Visible;
            if rwy_bar.is_fill {
                bar_sp.custom_size = Some(Vec2::new((BAR_WIDTH * prog).max(0.01), 6.0));
            }
        } else {
            *bar_vis = Visibility::Hidden;
            if rwy_bar.is_fill {
                bar_sp.custom_size = Some(Vec2::new(0.01, 6.0));
            }
        }
    }
}

fn update_hud(
    time: Res<Time>,
    mut gs: ResMut<GameState>,
    mut timer_q:    Query<(&mut Text, &mut TextColor), (With<TimerText>, Without<MoneyText>, Without<TruckUpgradeText>, Without<PlaneCountText>, Without<LevelCompleteText>, Without<TruckUpgradeBtnText>, Without<TruckIconFuel>, Without<UpgTruckFuel>, Without<WorldTruckFuel>)>,
    mut money_q:    Query<&mut Text, (With<MoneyText>, Without<TimerText>, Without<TruckUpgradeText>, Without<PlaneCountText>, Without<LevelCompleteText>, Without<TruckUpgradeBtnText>)>,
    mut truck_q:    Query<&mut Text, (With<TruckUpgradeText>, Without<TimerText>, Without<MoneyText>, Without<PlaneCountText>, Without<LevelCompleteText>, Without<TruckUpgradeBtnText>)>,
    mut lc_q:       Query<(&mut Text, &mut TextColor), (With<LevelCompleteText>, Without<TruckUpgradeBtnText>, Without<TruckIconFuel>, Without<UpgTruckFuel>, Without<WorldTruckFuel>)>,
    mut crash_q:    Query<&mut TextColor, (With<CrashText>, Without<LevelCompleteText>, Without<TimerText>, Without<TruckIconFuel>, Without<UpgTruckFuel>, Without<WorldTruckFuel>)>,
    mut restart_q:   Query<(&mut Visibility, &mut BackgroundColor), (With<RestartButton>, Without<ProceedButton>, Without<TruckUpgradeButton>, Without<TruckIconBody>, Without<TruckIconCab>, Without<UpgTruckBody>, Without<UpgTruckCab>)>,
    mut proceed_q:   Query<(&mut Visibility, &mut BackgroundColor), (With<ProceedButton>, Without<RestartButton>, Without<TruckUpgradeButton>, Without<TruckIconBody>, Without<TruckIconCab>, Without<UpgTruckBody>, Without<UpgTruckCab>)>,
    mut upg_btn_q:   Query<&mut BackgroundColor, (With<TruckUpgradeButton>, Without<RestartButton>, Without<ProceedButton>, Without<TruckIconBody>, Without<TruckIconCab>, Without<UpgTruckBody>, Without<UpgTruckCab>)>,
    mut upg_txt_q:   Query<&mut Text, (With<TruckUpgradeBtnText>, Without<TimerText>, Without<MoneyText>, Without<TruckUpgradeText>, Without<PlaneCountText>, Without<LevelCompleteText>)>,
    mut icon_body_q: Query<&mut BackgroundColor, (With<TruckIconBody>, Without<RestartButton>, Without<ProceedButton>, Without<TruckUpgradeButton>, Without<TruckIconCab>, Without<UpgTruckBody>, Without<UpgTruckCab>)>,
    mut icon_cab_q:  Query<&mut BackgroundColor, (With<TruckIconCab>,  Without<RestartButton>, Without<ProceedButton>, Without<TruckUpgradeButton>, Without<TruckIconBody>, Without<UpgTruckBody>, Without<UpgTruckCab>)>,
    mut upg_body_q:  Query<&mut BackgroundColor, (With<UpgTruckBody>,  Without<RestartButton>, Without<ProceedButton>, Without<TruckUpgradeButton>, Without<TruckIconBody>, Without<TruckIconCab>, Without<UpgTruckCab>)>,
    mut upg_cab_q:   Query<&mut BackgroundColor, (With<UpgTruckCab>,   Without<RestartButton>, Without<ProceedButton>, Without<TruckUpgradeButton>, Without<TruckIconBody>, Without<TruckIconCab>, Without<UpgTruckBody>)>,
) {
    gs.payout_flash -= time.delta_secs();

    // ── Crash overlay ─────────────────────────────────────────────────────
    if let Ok(mut c) = crash_q.get_single_mut() {
        *c = if gs.crashed { TextColor(Color::srgba(1.0, 0.2, 0.2, 1.0)) }
             else          { TextColor(Color::srgba(1.0, 0.2, 0.2, 0.0)) };
    }

    // ── Congratulations overlay ───────────────────────────────────────────
    if let Ok((mut text, mut color)) = lc_q.get_single_mut() {
        if gs.game_complete {
            *text  = Text::new(format!(
                "CONGRATULATIONS!\nAirport Ramp Chaos - CLEARED!\n{} planes fueled   ${:.0} earned\n\nClick RESTART to play again",
                WIN_PLANES, gs.money));
            *color = TextColor(Color::srgba(1.0, 0.88, 0.12, 1.0)); // gold
        } else {
            *color = TextColor(Color::srgba(0.1, 1.0, 0.35, 0.0));
        }
    }
    if let Ok((mut vis, _bg)) = proceed_q.get_single_mut() {
        *vis = Visibility::Hidden;
    }
    // Restart button also appears on game_complete so the player can play again
    if let Ok((mut vis, mut bg)) = restart_q.get_single_mut() {
        if !gs.started {
            *vis = Visibility::Visible; *bg = BackgroundColor(Color::srgb(0.1, 0.35, 0.9));
        } else if gs.crashed || gs.game_complete {
            *vis = Visibility::Visible; *bg = BackgroundColor(Color::srgb(0.1, 0.35, 0.9));
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // ── Payout / info flash (no countdown shown) ───────────────────────────
    if let Ok((mut text, mut color)) = timer_q.get_single_mut() {
        if !gs.started {
            *text  = Text::new("Fuel every plane before the next one lands!");
            *color = TextColor(Color::srgb(0.85, 0.85, 0.85));
        } else if gs.crashed || gs.game_complete {
            *text = Text::new("");
        } else if gs.payout_flash > 0.0 {
            *text  = Text::new(format!("FUELED!  +${:.0}   ({}/{})", gs.last_payout, gs.planes_fueled, WIN_PLANES));
            *color = TextColor(Color::srgb(0.15, 1.0, 0.45));
        } else {
            *text  = Text::new(format!("{}/{} planes fueled", gs.planes_fueled, WIN_PLANES));
            *color = TextColor(Color::srgb(0.78, 0.80, 0.84));
        }
    }

    // ── Top bar: money & truck level ──────────────────────────────────────
    if let Ok(mut t) = money_q.get_single_mut() {
        *t = Text::new(format!("${:.2}", gs.money));
    }
    if let Ok(mut t) = truck_q.get_single_mut() {
        let lvl = gs.truck_level;
        *t = Text::new(if lvl < 5 { format!("LV {}", lvl) } else { "MAX".into() });
    }

    // ── Upgrade button text + affordability colour ─────────────────────────
    let lvl  = gs.truck_level;
    let cost = if lvl < 5 { TRUCK_UPGRADE_COSTS[lvl as usize] } else { 0.0 };

    if let Ok(mut t) = upg_txt_q.get_single_mut() {
        *t = Text::new(if lvl >= 5 { "MAX".into() } else { format!("${:.0}", cost) });
    }
    if let Ok(mut bg) = upg_btn_q.get_single_mut() {
        *bg = BackgroundColor(if lvl >= 5 {
            Color::srgb(0.28, 0.28, 0.30)
        } else if gs.money >= cost {
            Color::srgb(0.95, 0.55, 0.10)
        } else {
            Color::srgb(0.45, 0.30, 0.15)
        });
    }

    // ── Truck icons ────────────────────────────────────────────────────────
    // Bar icon = current level; button icon = next level
    let (cur_t, cur_c) = truck_level_colors(gs.truck_level);
    let (upg_t, upg_c) = truck_next_level_colors(gs.truck_level);
    if let Ok(mut bg) = icon_body_q.get_single_mut() { *bg = BackgroundColor(cur_t); }
    if let Ok(mut bg) = icon_cab_q.get_single_mut()  { *bg = BackgroundColor(cur_c); }
    if let Ok(mut bg) = upg_body_q.get_single_mut()  { *bg = BackgroundColor(upg_t); }
    if let Ok(mut bg) = upg_cab_q.get_single_mut()   { *bg = BackgroundColor(upg_c); }
}

fn update_world_fuel_label(
    truck_q: Query<&Transform, (With<FuelTruck>, Without<WorldTruckFuel>)>,
    mut fuel_q: Query<&mut Transform, (With<WorldTruckFuel>, Without<FuelTruck>)>,
) {
    let Ok(truck_tf) = truck_q.get_single() else { return };
    let Ok(mut fuel_tf) = fuel_q.get_single_mut() else { return };
    fuel_tf.translation.x = truck_tf.translation.x;
    fuel_tf.translation.y = truck_tf.translation.y;
    fuel_tf.rotation = Quat::IDENTITY;
}

fn update_fuel_labels(
    gs: Res<GameState>,
    mut fuel_txt_q:   Query<&mut TextColor, (With<TruckIconFuel>, Without<UpgTruckFuel>, Without<WorldTruckFuel>)>,
    mut upg_fuel_q:   Query<&mut TextColor, (With<UpgTruckFuel>, Without<TruckIconFuel>, Without<WorldTruckFuel>)>,
    mut world_fuel_q: Query<&mut TextColor, (With<WorldTruckFuel>, Without<TruckIconFuel>, Without<UpgTruckFuel>)>,
) {
    if let Ok(mut tc) = fuel_txt_q.get_single_mut() {
        *tc = TextColor(truck_label_color(gs.truck_level));
    }
    if let Ok(mut tc) = upg_fuel_q.get_single_mut() {
        *tc = TextColor(truck_label_color(gs.truck_level + 1));
    }
    if let Ok(mut tc) = world_fuel_q.get_single_mut() {
        *tc = TextColor(truck_label_color(gs.truck_level));
    }
}

fn update_truck_label(
    gs: Res<GameState>,
    truck_q: Query<&Transform, With<FuelTruck>>,
    mut label_q: Query<(&mut Transform, &mut TextColor), (With<PlaneCountText>, Without<FuelTruck>)>,
) {
    let Ok(tf) = truck_q.get_single() else { return };
    let Ok((mut l, mut tc)) = label_q.get_single_mut() else { return };
    l.translation.x = tf.translation.x;
    l.translation.y = tf.translation.y;
    l.translation.z = 5.0;
    *tc = TextColor(truck_label_color(gs.truck_level));
}

fn handle_upgrades(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut gs: ResMut<GameState>,
    mut commands: Commands,
    tr_q:   Query<(Entity, &Transform), With<FuelTruck>>,
    fuel_q: Query<Entity, With<WorldTruckFuel>>,
    btn_q:  Query<&Interaction, With<TruckUpgradeButton>>,
) {
    if gs.truck_level >= 5 { return; }
    let key_pressed = keys.just_pressed(KeyCode::KeyU);
    let btn_clicked = mouse.just_pressed(MouseButton::Left)
        && btn_q.get_single().map(|i| *i == Interaction::Pressed).unwrap_or(false);
    if !key_pressed && !btn_clicked { return; }

    let cost = TRUCK_UPGRADE_COSTS[gs.truck_level as usize];
    if gs.money >= cost {
        gs.money -= cost;
        gs.truck_level += 1;
        if let Ok((entity, tf)) = tr_q.get_single() {
            let pos = tf.translation.truncate();
            commands.entity(entity).despawn_recursive();
            for e in &fuel_q { commands.entity(e).despawn_recursive(); }
            spawn_truck(&mut commands, gs.truck_level, pos);
        }
    }
}

fn handle_restart(
    mouse: Res<ButtonInput<MouseButton>>,
    mut gs: ResMut<GameState>,
    mut commands: Commands,
    ac_q:        Query<Entity, With<Aircraft>>,
    tr_q:        Query<Entity, With<FuelTruck>>,
    fuel_q:      Query<Entity, With<WorldTruckFuel>>,
    particle_q:  Query<Entity, With<Particle>>,
    mut bar_q:   Query<(&RunwayFuelBar, &mut Sprite, &mut Visibility)>,
    btn_q:       Query<&Interaction, With<RestartButton>>,
    mut text_q:  Query<&mut Text, With<RestartButtonText>>,
) {
    let should_act = !gs.started || gs.crashed || gs.game_complete;
    if !should_act { return; }

    let clicked = mouse.just_pressed(MouseButton::Left)
        && btn_q.get_single().map(|i| *i == Interaction::Pressed).unwrap_or(false);
    if !clicked { return; }

    if !gs.started {
        // First click — start the game, swap label to RESTART
        gs.started = true;
        if let Ok(mut t) = text_q.get_single_mut() { *t = Text::new("RESTART"); }
    } else {
        // Crash restart — full reset but keep game started
        for e in &ac_q       { commands.entity(e).despawn_recursive(); }
        for e in &tr_q       { commands.entity(e).despawn_recursive(); }
        for e in &fuel_q     { commands.entity(e).despawn_recursive(); }
        for e in &particle_q { commands.entity(e).despawn_recursive(); }

        *gs = GameState::default();
        gs.started = true;
        spawn_truck(&mut commands, gs.truck_level, TRUCK_START);
        for (_, mut bar_sp, mut bar_vis) in &mut bar_q {
            *bar_vis = Visibility::Hidden;
            bar_sp.custom_size = Some(Vec2::new(0.01, 6.0));
        }
    }
}

fn spawn_crash_effects(
    mut gs: ResMut<GameState>,
    mut commands: Commands,
) {
    if !gs.crashed || gs.crash_anim_spawned { return; }
    gs.crash_anim_spawned = true;

    let pos = gs.crash_pos;

    // Explosion flash — large, grows fast, fades
    commands.spawn((
        Particle { velocity: Vec2::ZERO, lifetime: 0.0, max_lifetime: 0.55,
                   r: 1.0, g: 0.92, b: 0.3, scale_start: 0.4, scale_end: 2.8 },
        Sprite { color: Color::srgba(1.0, 0.92, 0.3, 1.0), custom_size: Some(Vec2::splat(70.0)), ..default() },
        Transform::from_xyz(pos.x, pos.y, 10.0),
    ));

    // Fire particles — rise, shrink, fade
    for i in 0..16usize {
        let a = i as f32 * 0.628 + 0.2;
        let spread = 15.0 + (i as f32 * 11.3) % 30.0;
        let rise   = 35.0 + (i as f32 *  8.7) % 35.0;
        let vel = Vec2::new(a.cos() * spread * 0.5, a.sin().abs() * spread + rise);
        let size = 9.0 + (i as f32 * 3.7) % 11.0;
        let ml   = 0.55 + (i as f32 * 0.031) % 0.45;
        let (r, g, b) = match i % 3 {
            0 => (1.0_f32, 0.88, 0.12),
            1 => (1.0,    0.42, 0.04),
            _ => (0.88,   0.12, 0.03),
        };
        commands.spawn((
            Particle { velocity: vel, lifetime: 0.0, max_lifetime: ml,
                       r, g, b, scale_start: 1.0, scale_end: 0.2 },
            Sprite { color: Color::srgba(r, g, b, 1.0), custom_size: Some(Vec2::splat(size)), ..default() },
            Transform::from_xyz(pos.x + a.cos() * 8.0, pos.y + a.sin() * 8.0, 9.0),
        ));
    }

    // Smoke particles — drift up, expand, fade slowly
    for i in 0..12usize {
        let a = i as f32 * 0.785 + 0.4;
        let drift = (i as f32 * 6.9) % 18.0 - 9.0;
        let rise  = 20.0 + (i as f32 * 5.3) % 22.0;
        let vel = Vec2::new(drift, rise);
        let size = 16.0 + (i as f32 * 4.1) % 18.0;
        let ml   = 1.1 + (i as f32 * 0.065) % 0.9;
        let gray = 0.22 + (i as f32 * 0.025) % 0.22;
        commands.spawn((
            Particle { velocity: vel, lifetime: 0.0, max_lifetime: ml,
                       r: gray, g: gray, b: gray, scale_start: 0.7, scale_end: 2.2 },
            Sprite { color: Color::srgba(gray, gray, gray, 0.8), custom_size: Some(Vec2::splat(size)), ..default() },
            Transform::from_xyz(pos.x + a.cos() * 12.0, pos.y + a.sin() * 6.0 + 8.0, 8.0),
        ));
    }
}

fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut p, mut tf, mut sp) in &mut q {
        p.lifetime += time.delta_secs();
        if p.lifetime >= p.max_lifetime {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        let t = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
        tf.translation += (p.velocity * time.delta_secs()).extend(0.0);
        tf.scale = Vec3::splat(p.scale_start + (p.scale_end - p.scale_start) * t);
        sp.color = Color::srgba(p.r, p.g, p.b, (1.0 - t).max(0.0));
    }
}

fn update_active_runways(
    gs: Res<GameState>,
    mut label_q: Query<(&ComingSoonLabel, &mut Visibility)>,
    mut dash_q:  Query<(&VerticalRunwayDash, &mut Sprite), Without<ParkingStandSide>>,
    mut stand_q: Query<(&ParkingStandSide, &mut Sprite), Without<VerticalRunwayDash>>,
) {
    for (label, mut v) in &mut label_q {
        let open = (label.0 as usize) < gs.active_runways;
        *v = if open { Visibility::Hidden } else { Visibility::Visible };
    }
    for (dash, mut sp) in &mut dash_q {
        let open = (dash.0 as usize) < gs.active_runways;
        sp.color = if open { Color::srgb(0.98, 0.88, 0.08) }    // yellow
                   else    { Color::srgb(0.35, 0.35, 0.36) };   // gray
    }
    for (stand, mut sp) in &mut stand_q {
        let open = (stand.0 as usize) < gs.active_runways;
        sp.color = if open { Color::srgb(0.95, 0.95, 0.95) }    // white
                   else    { Color::srgb(0.30, 0.30, 0.30) };   // dim
    }
}

fn spawn_fireworks(
    time: Res<Time>,
    mut gs: ResMut<GameState>,
    mut commands: Commands,
) {
    if !gs.game_complete { return; }
    gs.firework_timer -= time.delta_secs();
    if gs.firework_timer > 0.0 { return; }
    gs.firework_timer = 0.45;

    // Pick a random burst centre across the screen
    let cx = (gs.next_rand() - 0.5) * 900.0;
    let cy = (gs.next_rand() - 0.5) * 500.0;
    let hue = gs.next_rand();
    let (r, g, b) = match (hue * 6.0) as u8 {
        0 => (1.0_f32, 0.25, 0.25),
        1 => (1.0,    0.65, 0.15),
        2 => (1.0,    0.95, 0.20),
        3 => (0.30,   1.0,  0.35),
        4 => (0.25,   0.65, 1.0),
        _ => (0.85,   0.35, 1.0),
    };

    // Central flash
    commands.spawn((
        Particle { velocity: Vec2::ZERO, lifetime: 0.0, max_lifetime: 0.35,
                   r, g, b, scale_start: 0.5, scale_end: 1.6 },
        Sprite { color: Color::srgba(r, g, b, 1.0), custom_size: Some(Vec2::splat(22.0)), ..default() },
        Transform::from_xyz(cx, cy, 10.0),
    ));

    // Radial burst
    for i in 0..22usize {
        let a = i as f32 * 0.2856; // ~2π/22
        let speed = 110.0 + (i as f32 * 7.3) % 60.0;
        let vel = Vec2::new(a.cos() * speed, a.sin() * speed);
        let ml  = 0.9 + (i as f32 * 0.037) % 0.6;
        commands.spawn((
            Particle { velocity: vel, lifetime: 0.0, max_lifetime: ml,
                       r, g, b, scale_start: 1.0, scale_end: 0.2 },
            Sprite { color: Color::srgba(r, g, b, 1.0), custom_size: Some(Vec2::splat(7.0)), ..default() },
            Transform::from_xyz(cx, cy, 9.5),
        ));
    }
}

fn update_cursor_icon(
    mut commands: Commands,
    btn_q: Query<&Interaction, With<Button>>,
    win_q: Query<Entity, With<PrimaryWindow>>,
) {
    let Ok(window) = win_q.get_single() else { return };
    let hovered = btn_q.iter().any(|i| matches!(i, Interaction::Hovered | Interaction::Pressed));
    let icon = if hovered { SystemCursorIcon::Pointer } else { SystemCursorIcon::Default };
    commands.entity(window).insert(CursorIcon::System(icon));
}
