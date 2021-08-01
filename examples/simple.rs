use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::prelude::random;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(PanCamPlugin::default())
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PanCam::default());

    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let size = Vec2::new(spacing, spacing);
    for x in 0..n {
        for y in 0..n {
            let x = x as f32 * spacing - offset;
            let y = y as f32 * spacing - offset;
            let color = Color::hsl(240., random::<f32>() * 0.3, random::<f32>() * 0.3);
            commands.spawn_bundle(SpriteBundle {
                material: materials.add(color.into()),
                sprite: Sprite {
                    size,
                    ..Default::default()
                },
                transform: Transform::from_xyz(x, y, 0.),
                ..Default::default()
            });
        }
    }
}