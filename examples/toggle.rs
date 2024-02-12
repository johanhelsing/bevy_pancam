use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamBundle, PanCamPlugin};
use rand::prelude::random;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_key)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), PanCamBundle::default()));

    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    for x in 0..n {
        for y in 0..n {
            let x = x as f32 * spacing - offset;
            let y = y as f32 * spacing - offset;
            let color = Color::hsl(240., random::<f32>() * 0.3, random::<f32>() * 0.3);
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size,
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.),
                ..default()
            });
        }
    }
}

fn toggle_key(mut query: Query<&mut PanCam>, keys: Res<ButtonInput<KeyCode>>) {
    // Space = Toggle Panning
    if keys.just_pressed(KeyCode::Space) {
        for mut pancam in &mut query {
            pancam.enabled = !pancam.enabled;
        }
    }
    // T = Toggle Zoom to Cursor
    if keys.just_pressed(KeyCode::KeyT) {
        for mut pancam in &mut query {
            pancam.zoom_to_cursor = !pancam.zoom_to_cursor;
        }
    }
}
