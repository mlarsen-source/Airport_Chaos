use bevy::prelude::*;

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
        .add_systems(Startup, setup)
        .add_systems(Update, (taxi_aircraft, move_fuel_truck, handle_click))
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

    // Runway strip along the top edge
    commands.spawn((
        Sprite {
            color: Color::srgb(0.15, 0.15, 0.15),
            custom_size: Some(Vec2::new(900.0, 80.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 285.0, 1.0),
    ));

    // Aircraft spawns on runway and taxis to parking stand
    commands.spawn((
        Aircraft {
            target: Vec2::new(100.0, 100.0),
            speed: 80.0,
            parked: false,
        },
        Sprite {
            color: Color::srgb(0.9, 0.9, 0.9),
            custom_size: Some(Vec2::new(80.0, 30.0)),
            ..default()
        },
        Transform::from_xyz(100.0, 360.0, 2.0),
    ));

    // Fuel truck - player controlled
    commands.spawn((
        FuelTruck {
            destination: None,
            speed: 120.0,
        },
        Sprite {
            color: Color::srgb(0.95, 0.8, 0.1),
            custom_size: Some(Vec2::new(30.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(-250.0, -150.0, 2.0),
    ));
}

fn taxi_aircraft(
    time: Res<Time>,
    mut query: Query<(&mut Aircraft, &mut Transform, &mut Sprite)>,
) {
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
            let delta = direction * aircraft.speed * time.delta_secs();
            transform.translation += delta.extend(0.0);
        }
    }
}

fn handle_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut truck_q: Query<&mut FuelTruck>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            for mut truck in &mut truck_q {
                truck.destination = Some(world_pos);
            }
        }
    }
}

fn move_fuel_truck(
    time: Res<Time>,
    mut query: Query<(&mut FuelTruck, &mut Transform)>,
) {
    for (mut truck, mut transform) in &mut query {
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
            let delta = direction * truck.speed * time.delta_secs();
            transform.translation += delta.extend(0.0);
        }
    }
}
