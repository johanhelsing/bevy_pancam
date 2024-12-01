use bevy::{prelude::*, render::camera::Viewport};
use bevy_pancam::{PanCam, PanCamPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(Viewport {
                physical_position: UVec2::new(100, 200),
                physical_size: UVec2::new(600, 400),
                depth: 0.0..1.0,
            }),
            ..default()
        },
        PanCam {
            min_x: -500.,
            min_y: -500.,
            max_x: 500.,
            max_y: 500.,
            ..default()
        },
    ));

    // background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(1000., 1000.)),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.),
    ));

    // red square
    commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.3, 0.3),
            custom_size: Some(Vec2::new(100., 100.)),
            ..default()
        },
        Transform::from_xyz(0., 0., 1.),
    ));
}
