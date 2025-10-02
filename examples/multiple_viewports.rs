use bevy::{
    camera::{ScalingMode, Viewport},
    prelude::*,
    window::WindowResized,
};
use bevy_pancam::{PanCam, PanCamClampBounds, PanCamPlugin};
use rand::random;

#[derive(Component)]
#[require(Camera2d)]
struct LeftCamera;

#[derive(Component)]
struct LeftPanel;

#[derive(Component)]
#[require(Camera2d)]
struct RightCamera;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, reset_viewports)
        .run();
}

fn setup(mut commands: Commands) {
    // left camera is fixed, draws some simple GUI.
    let left_panel_camera = commands
        .spawn((
            LeftCamera,
            Camera {
                order: 0,
                ..default()
            },
            Transform::from_xyz(-10000., -10000., 0.),
        ))
        .id();
    commands.spawn((
        LeftPanel,
        UiTargetCamera(left_panel_camera),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        Text::new("In this example, the pattern of squares on the right will always fill 80% of the window, keeping its aspect ratio, even as the window resizes."),
    ));

    commands.spawn((
        RightCamera,
        Camera {
            order: 1,
            ..default()
        },
        // constrain the bounds to EXACTLY the world area that holds the demo sprites
        PanCam {
            min_x: -525.,
            max_x: 475.,
            min_y: -525.,
            max_y: 475.,
            ..default()
        },
        // use AutoMax scaling on the projection itself. this MAY only be needed to work around #73.
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax {
                max_width: 1000.,
                max_height: 1000.,
            },
            ..OrthographicProjection::default_2d()
        }),
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

fn reset_viewports(
    windows: Query<&Window>,
    mut resize_events: MessageReader<WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<(Entity, &mut Camera), (Without<LeftCamera>, With<RightCamera>)>,
    mut commands: Commands,
) -> Result {
    let mut l = left_camera.single_mut()?;
    let (entity, mut r) = right_camera.single_mut()?;
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window)?;
        let size = window.physical_size();

        let sep = (size.x + 4) / 5;
        l.viewport = Some(Viewport {
            physical_size: UVec2 { x: sep, y: size.y },
            ..default()
        });

        r.viewport = Some(Viewport {
            physical_position: UVec2 { x: sep, y: 0 },
            physical_size: UVec2 {
                x: size.x - sep,
                y: size.y,
            },
            ..default()
        });

        // for this kind of thing to work properly, we must manually trigger bevy_pancam to clamp
        // the bounds, since it only does this automatically when it's the one to move them.
        commands.trigger(PanCamClampBounds { entity });
    }

    Ok(())
}
