use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamAction, PanCamPlugin};
use leafwing_input_manager::{
    action_state::ActionState,
    axislike::SingleAxis,
    input_map::InputMap,
    user_input::{InputKind, Modifier},
    InputManagerBundle,
};
use rand::prelude::random;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let mut input_map = InputMap::default();
    input_map.insert_chord(
        [
            InputKind::Modifier(Modifier::Alt),
            InputKind::Mouse(MouseButton::Left),
        ],
        PanCamAction::Grab,
    );
    input_map.insert(SingleAxis::mouse_wheel_y(), PanCamAction::Zoom);
    commands.spawn((
        Camera2dBundle::default(),
        PanCam::default(),
        InputManagerBundle::<PanCamAction> {
            action_state: ActionState::default(),
            input_map,
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
