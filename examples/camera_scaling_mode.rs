use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_pancam::{PanCam, PanCamBundle, PanCamPlugin};
use rand::prelude::random;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(10.0);

    commands.spawn((
        cam,
        PanCamBundle {
            pan_cam: PanCam {
                min_x: Some(-10.),
                max_x: Some(10.),
                min_y: Some(-10.),
                max_y: Some(10.0),
                ..default()
            },
            ..default()
        },
    ));

    let n = 20;
    let spacing = 1.;
    let offset = -spacing * n as f32 / 2. + spacing / 2.;
    let custom_size = Some(Vec2::new(1.0, 1.0));
    for x in 0..n {
        for y in 0..n {
            let x = x as f32 * spacing + offset;
            let y = y as f32 * spacing + offset;
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
