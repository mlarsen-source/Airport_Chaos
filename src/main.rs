use bevy::prelude::*;

#[derive(Component)]
struct Aircraft {
    target: Vec2,
    speed: f32,
    parked: bool,
}

#[derive(Component)]
struct FuelTruck;

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
        .add_systems(Update, taxi_aircraft)
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

    // Fuel truck starts parked near the terminal
    commands.spawn((
        FuelTruck,
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
            // Turn blue when parked to show it's ready for service
            sprite.color = Color::srgb(0.6, 0.7, 0.95);
        } else {
            let direction = diff.normalize();
            // Rotate to face direction of travel
            transform.rotation =
                Quat::from_rotation_z(direction.to_angle() - std::f32::consts::FRAC_PI_2);
            let delta = direction * aircraft.speed * time.delta_secs();
            transform.translation += delta.extend(0.0);
        }
    }
}
