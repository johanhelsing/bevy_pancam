use bevy::prelude::*;
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
        PanCam {
            // prevent the camera from zooming too far out
            max_scale: 40.,
            // prevent the camera from zooming too far in
            min_scale: 0.5,
            // prevent the camera from panning or zooming past x -750. -750 was chosen to display both the edges of the
            // sample image and some boundary space beyond it
            min_x: -750.,
            // prevent the camera from panning or zooming past x 1750. 1750 was chosen to show an asymmetric boundary
            // window with more space on the right than the left
            max_x: 1750.,
            // prevent the camera from panning or zooming past y -750. value chosen for same reason as above
            min_y: -750.,
            // prevent the camera from panning or zooming past y 1750. value chosen for same reason as above
            max_y: 1750.,
            ..default()
        },
    ));

    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    for x in 0..n {
        for y in 0..n {
            let x = x as f32 * spacing - offset;
            let y = y as f32 * spacing - offset;
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
