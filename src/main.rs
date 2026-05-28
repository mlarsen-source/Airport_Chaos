use bevy::prelude::*;

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

    // Aircraft (white rectangle pointing right, parked at stand)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.9, 0.9, 0.9),
            custom_size: Some(Vec2::new(80.0, 30.0)),
            ..default()
        },
        Transform::from_xyz(100.0, 150.0, 2.0),
    ));

    // Fuel truck (yellow rectangle)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.95, 0.8, 0.1),
            custom_size: Some(Vec2::new(30.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(-200.0, -100.0, 2.0),
    ));
}
