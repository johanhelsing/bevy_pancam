use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::camera::OrthographicProjection,
};

#[derive(Default)]
pub struct PanCamPlugin;

impl Plugin for PanCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_movement).add_system(camera_zoom);
    }
}

// Zoom doesn't work on bevy 0.5 due to: https://github.com/bevyengine/bevy/pull/2015
fn camera_zoom(
    mut query: Query<(&PanCam, &mut OrthographicProjection, &mut Transform)>,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Res<Windows>,
) {
    let pixels_per_line = 100.; // Maybe make configurable?
    let scroll = scroll_events
        .iter()
        .map(|ev| match ev.unit {
            MouseScrollUnit::Pixel => ev.y,
            MouseScrollUnit::Line => ev.y * pixels_per_line,
        })
        .sum::<f32>();

    if scroll == 0. {
        return;
    }

    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let mouse_normalized_screen_pos = (window.cursor_position().unwrap() / window_size) * 2. - Vec2::ONE;
    
    for (cam, mut proj, mut pos) in query.iter_mut() {
        if cam.enabled {
            let old_scale = proj.scale;
            proj.scale = (proj.scale * (1. + -scroll * 0.001)).max(0.00001);

            if cam.track_mouse {
                let proj_size = Vec2::new(proj.right, proj.top);
                let mouse_world_pos = pos.translation.truncate() + mouse_normalized_screen_pos * proj_size * old_scale;
                pos.translation = (mouse_world_pos - mouse_normalized_screen_pos * proj_size * proj.scale).extend(pos.translation.z);
            }
        }
    }
}

fn camera_movement(
    windows: Res<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut query: Query<(&PanCam, &mut Transform, &OrthographicProjection)>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let window = windows.get_primary().unwrap();

    // Use position instead of MouseMotion, otherwise we don't get acceleration movement
    let current_pos = match window.cursor_position() {
        Some(current_pos) => current_pos,
        None => return,
    };
    let delta = current_pos - last_pos.unwrap_or(current_pos);

    for (cam, mut transform, projection) in query.iter_mut() {
        if cam.enabled
            && cam
                .grab_buttons
                .iter()
                .any(|btn| mouse_buttons.pressed(*btn))
        {
            let scaling = Vec2::new(
                window.width() / (projection.right - projection.left),
                window.height() / (projection.top - projection.bottom),
            ) * projection.scale;

            transform.translation -= (delta * scaling).extend(0.);
        }
    }
    *last_pos = Some(current_pos);
}

#[derive(Component)]
pub struct PanCam {
    pub grab_buttons: Vec<MouseButton>,
    pub enabled: bool,
    pub track_mouse: bool,
}

impl Default for PanCam {
    fn default() -> Self {
        Self {
            grab_buttons: vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle],
            enabled: true,
            track_mouse: false,
        }
    }
}
