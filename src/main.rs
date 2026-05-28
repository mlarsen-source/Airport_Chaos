use bevy::prelude::*;

const AIRCRAFT_START: Vec2 = Vec2::new(100.0, 360.0);
const AIRCRAFT_PARK: Vec2 = Vec2::new(100.0, 100.0);
const TRUCK_START: Vec2 = Vec2::new(-250.0, -150.0);
const COLLISION_RADIUS: f32 = 48.0;
const FUEL_RADIUS: f32 = 60.0;
const FUEL_DURATION: f32 = 4.0;

#[derive(Component)]
struct Aircraft {
    target: Vec2,
    speed: f32,
    parked: bool,
}

#[derive(Component)]
struct FuelTruck {
    destination: Option<Vec2>,
    speed: f32,
    fueling: bool,
    fuel_timer: f32,
}

#[derive(Component)]
struct CrashText;

#[derive(Component)]
struct WinText;

#[derive(Component)]
struct FuelBarBg;

#[derive(Component)]
struct FuelBarFill;

#[derive(Resource, Default)]
struct GameState {
    crashed: bool,
    won: bool,
}

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
        .add_systems(
            Update,
            (
                taxi_aircraft,
                handle_click,
                move_fuel_truck,
                check_collision,
                check_fueling,
                handle_restart,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Apron background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.25, 0.27, 0.25),
            custom_size: Some(Vec2::new(900.0, 650.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Runway strip
    commands.spawn((
        Sprite {
            color: Color::srgb(0.15, 0.15, 0.15),
            custom_size: Some(Vec2::new(900.0, 80.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 285.0, 1.0),
    ));

    // Fuel bar background
    commands.spawn((
        FuelBarBg,
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.2),
            custom_size: Some(Vec2::new(204.0, 24.0)),
            ..default()
        },
        Transform::from_xyz(100.0, 55.0, 3.0),
    ));

    // Fuel bar fill (starts at zero width)
    commands.spawn((
        FuelBarFill,
        Sprite {
            color: Color::srgb(0.1, 0.8, 0.3),
            custom_size: Some(Vec2::new(0.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(100.0 - 100.0, 55.0, 4.0),
    ));

    spawn_aircraft(&mut commands);
    spawn_truck(&mut commands);

    // Crash overlay text
    commands.spawn((
        CrashText,
        Text::new("COLLISION!\nPress R to restart"),
        TextFont { font_size: 48.0, ..default() },
        TextColor(Color::srgba(0.9, 0.2, 0.2, 0.0)),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(35.0),
            left: Val::Percent(25.0),
            right: Val::Percent(25.0),
            ..default()
        },
    ));

    // Win overlay text
    commands.spawn((
        WinText,
        Text::new("FUELED!\nMission Complete\nPress R to play again"),
        TextFont { font_size: 48.0, ..default() },
        TextColor(Color::srgba(0.1, 0.9, 0.3, 0.0)),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(35.0),
            left: Val::Percent(25.0),
            right: Val::Percent(25.0),
            ..default()
        },
    ));
}

fn spawn_aircraft(commands: &mut Commands) {
    commands.spawn((
        Aircraft {
            target: AIRCRAFT_PARK,
            speed: 40.0,
            parked: false,
        },
        Sprite {
            color: Color::srgb(0.9, 0.9, 0.9),
            custom_size: Some(Vec2::new(80.0, 30.0)),
            ..default()
        },
        Transform::from_xyz(AIRCRAFT_START.x, AIRCRAFT_START.y, 2.0),
    ));
}

fn spawn_truck(commands: &mut Commands) {
    commands.spawn((
        FuelTruck {
            destination: None,
            speed: 120.0,
            fueling: false,
            fuel_timer: 0.0,
        },
        Sprite {
            color: Color::srgb(0.95, 0.8, 0.1),
            custom_size: Some(Vec2::new(30.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(TRUCK_START.x, TRUCK_START.y, 2.0),
    ));
}

fn taxi_aircraft(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut query: Query<(&mut Aircraft, &mut Transform, &mut Sprite)>,
) {
    if game_state.crashed || game_state.won {
        return;
    }
    for (mut aircraft, mut transform, mut sprite) in &mut query {
        if aircraft.parked {
            continue;
        }
        let current = transform.translation.truncate();
        let diff = aircraft.target - current;
        let distance = diff.length();

        if distance < 3.0 {
            transform.translation.x = aircraft.target.x;
            transform.translation.y = aircraft.target.y;
            aircraft.parked = true;
            sprite.color = Color::srgb(0.6, 0.7, 0.95);
        } else {
            let direction = diff.normalize();
            transform.rotation =
                Quat::from_rotation_z(direction.to_angle() - std::f32::consts::FRAC_PI_2);
            transform.translation += (direction * aircraft.speed * time.delta_secs()).extend(0.0);
        }
    }
}

fn handle_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    game_state: Res<GameState>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut truck_q: Query<&mut FuelTruck>,
) {
    if game_state.crashed || game_state.won || !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            for mut truck in &mut truck_q {
                if !truck.fueling {
                    truck.destination = Some(world_pos);
                }
            }
        }
    }
}

fn move_fuel_truck(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut query: Query<(&mut FuelTruck, &mut Transform)>,
) {
    if game_state.crashed || game_state.won {
        return;
    }
    for (mut truck, mut transform) in &mut query {
        if truck.fueling {
            continue;
        }
        let Some(dest) = truck.destination else { continue };
        let current = transform.translation.truncate();
        let diff = dest - current;
        let distance = diff.length();

        if distance < 3.0 {
            transform.translation.x = dest.x;
            transform.translation.y = dest.y;
            truck.destination = None;
        } else {
            let direction = diff.normalize();
            transform.rotation =
                Quat::from_rotation_z(direction.to_angle() - std::f32::consts::FRAC_PI_2);
            transform.translation += (direction * truck.speed * time.delta_secs()).extend(0.0);
        }
    }
}

fn check_collision(
    mut game_state: ResMut<GameState>,
    aircraft_q: Query<(&Transform, &Aircraft)>,
    mut truck_q: Query<(&Transform, &mut Sprite), With<FuelTruck>>,
    mut crash_text_q: Query<&mut TextColor, With<CrashText>>,
) {
    if game_state.crashed || game_state.won {
        return;
    }
    let Ok((aircraft_tf, aircraft)) = aircraft_q.get_single() else { return };
    if aircraft.parked {
        return;
    }
    let Ok((truck_tf, mut truck_sprite)) = truck_q.get_single_mut() else { return };

    let distance = aircraft_tf
        .translation
        .truncate()
        .distance(truck_tf.translation.truncate());

    if distance < COLLISION_RADIUS {
        game_state.crashed = true;
        truck_sprite.color = Color::srgb(0.9, 0.1, 0.1);
        if let Ok(mut color) = crash_text_q.get_single_mut() {
            *color = TextColor(Color::srgba(0.9, 0.2, 0.2, 1.0));
        }
    }
}

fn check_fueling(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    aircraft_q: Query<(&Transform, &Aircraft)>,
    mut truck_q: Query<(&Transform, &mut FuelTruck, &mut Sprite)>,
    mut bar_q: Query<(&mut Sprite, &mut Transform), (With<FuelBarFill>, Without<FuelTruck>)>,
    mut win_text_q: Query<&mut TextColor, With<WinText>>,
) {
    if game_state.crashed || game_state.won {
        return;
    }
    let Ok((aircraft_tf, aircraft)) = aircraft_q.get_single() else { return };
    if !aircraft.parked {
        return;
    }
    let Ok((truck_tf, mut truck, mut truck_sprite)) = truck_q.get_single_mut() else { return };

    let distance = aircraft_tf
        .translation
        .truncate()
        .distance(truck_tf.translation.truncate());

    if distance < FUEL_RADIUS && !truck.fueling {
        truck.fueling = true;
        truck.destination = None;
        truck_sprite.color = Color::srgb(0.1, 0.9, 0.4);
    }

    if truck.fueling {
        truck.fuel_timer += time.delta_secs();
        let progress = (truck.fuel_timer / FUEL_DURATION).clamp(0.0, 1.0);

        if let Ok((mut bar_sprite, mut bar_tf)) = bar_q.get_single_mut() {
            let width = 200.0 * progress;
            bar_sprite.custom_size = Some(Vec2::new(width, 20.0));
            bar_tf.translation.x = AIRCRAFT_PARK.x - 100.0 + width / 2.0;
        }

        if truck.fuel_timer >= FUEL_DURATION {
            game_state.won = true;
            if let Ok(mut color) = win_text_q.get_single_mut() {
                *color = TextColor(Color::srgba(0.1, 0.9, 0.3, 1.0));
            }
        }
    }
}

fn handle_restart(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    aircraft_q: Query<Entity, With<Aircraft>>,
    truck_q: Query<Entity, With<FuelTruck>>,
    mut crash_text_q: Query<&mut TextColor, (With<CrashText>, Without<WinText>)>,
    mut win_text_q: Query<&mut TextColor, (With<WinText>, Without<CrashText>)>,
    mut bar_q: Query<(&mut Sprite, &mut Transform), With<FuelBarFill>>,
) {
    if !keys.just_pressed(KeyCode::KeyR) || (!game_state.crashed && !game_state.won) {
        return;
    }

    for e in &aircraft_q { commands.entity(e).despawn(); }
    for e in &truck_q { commands.entity(e).despawn(); }

    spawn_aircraft(&mut commands);
    spawn_truck(&mut commands);

    game_state.crashed = false;
    game_state.won = false;

    if let Ok(mut color) = crash_text_q.get_single_mut() {
        *color = TextColor(Color::srgba(0.9, 0.2, 0.2, 0.0));
    }
    if let Ok(mut color) = win_text_q.get_single_mut() {
        *color = TextColor(Color::srgba(0.1, 0.9, 0.3, 0.0));
    }
    if let Ok((mut bar_sprite, mut bar_tf)) = bar_q.get_single_mut() {
        bar_sprite.custom_size = Some(Vec2::new(0.0, 20.0));
        bar_tf.translation.x = AIRCRAFT_PARK.x - 100.0;
    }
}
