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
        .run();
}
