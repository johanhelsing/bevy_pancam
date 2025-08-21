use bevy::{camera::ScalingMode, prelude::*};
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::random;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 10.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        PanCam {
            min_x: -10.,
            max_x: 10.,
            min_y: -10.,
            max_y: 10.0,
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
            commands.spawn((
                Sprite {
                    color,
                    custom_size,
                    ..default()
                },
                Transform::from_xyz(x, y, 0.),
            ));
        }
    }
}
