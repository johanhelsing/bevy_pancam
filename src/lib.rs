#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy::{math::vec2, prelude::*, render::camera::CameraProjection, window::PrimaryWindow};

#[cfg(not(feature = "leafwing-input-manager"))]
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

#[cfg(feature = "leafwing-input-manager")]
use leafwing_input_manager::{
    action_state::ActionState, axislike::SingleAxis, input_map::InputMap,
    plugin::InputManagerPlugin, Actionlike, InputManagerBundle,
};

/// Plugin that adds the necessary systems for `PanCam` components to work
#[derive(Default)]
pub struct PanCamPlugin;

/// System set to allow ordering of `PanCamPlugin`
#[derive(Debug, Clone, Copy, SystemSet, PartialEq, Eq, Hash)]
pub struct PanCamSystemSet;

impl Plugin for PanCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (camera_movement, camera_zoom).in_set(PanCamSystemSet),
        )
        .register_type::<PanCam>();

        #[cfg(feature = "bevy_egui")]
        {
            app.init_resource::<EguiWantsFocus>()
                .add_systems(PostUpdate, check_egui_wants_focus)
                .configure_sets(
                    Update,
                    PanCamSystemSet.run_if(resource_equals(EguiWantsFocus(false))),
                );
        }

        #[cfg(feature = "leafwing-input-manager")]
        {
            app.add_plugins(InputManagerPlugin::<PanCamAction>::default());
        }
    }
}

#[derive(Resource, Deref, DerefMut, PartialEq, Eq, Default)]
#[cfg(feature = "bevy_egui")]
struct EguiWantsFocus(bool);

// todo: make run condition when Bevy supports mutable resources in them
#[cfg(feature = "bevy_egui")]
fn check_egui_wants_focus(
    mut contexts: Query<&mut bevy_egui::EguiContext>,
    mut wants_focus: ResMut<EguiWantsFocus>,
) {
    let ctx = contexts.iter_mut().next();
    let new_wants_focus = if let Some(ctx) = ctx {
        let ctx = ctx.into_inner().get_mut();
        ctx.wants_pointer_input() || ctx.wants_keyboard_input()
    } else {
        false
    };
    wants_focus.set_if_neq(EguiWantsFocus(new_wants_focus));
}

fn camera_zoom(
    mut query: Query<(&PanCam, &mut OrthographicProjection, &mut Transform)>,
    #[cfg(not(feature = "leafwing-input-manager"))] mut scroll_events: EventReader<MouseWheel>,
    #[cfg(feature = "leafwing-input-manager")] action_query: Query<&ActionState<PanCamAction>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    #[cfg(not(feature = "leafwing-input-manager"))]
    let scroll = {
        let pixels_per_line = 100.; // Maybe make configurable?
        scroll_events
            .read()
            .map(|ev| match ev.unit {
                MouseScrollUnit::Pixel => ev.y,
                MouseScrollUnit::Line => ev.y * pixels_per_line,
            })
            .sum::<f32>()
    };

    #[cfg(feature = "leafwing-input-manager")]
    let scroll = if let Ok(action_state) = action_query.get_single() {
        action_state.value(&PanCamAction::Zoom)
    } else {
        0.
    };

    if scroll == 0. {
        return;
    }

    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());
    let mouse_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2. - Vec2::ONE)
        .map(|p| Vec2::new(p.x, -p.y));

    for (cam, mut proj, mut pos) in &mut query {
        if cam.enabled {
            let old_scale = proj.scale;
            proj.scale = (proj.scale * (1. + -scroll * 0.001)).max(cam.min_scale);

            // Apply max scale constraint
            if let Some(max_scale) = cam.max_scale {
                proj.scale = proj.scale.min(max_scale);
            }

            // If there is both a min and max boundary, that limits how far we can zoom. Make sure we don't exceed that
            let scale_constrained = BVec2::new(
                cam.min_x.is_some() && cam.max_x.is_some(),
                cam.min_y.is_some() && cam.max_y.is_some(),
            );

            if scale_constrained.x || scale_constrained.y {
                let bounds_width = if let (Some(min_x), Some(max_x)) = (cam.min_x, cam.max_x) {
                    max_x - min_x
                } else {
                    f32::INFINITY
                };

                let bounds_height = if let (Some(min_y), Some(max_y)) = (cam.min_y, cam.max_y) {
                    max_y - min_y
                } else {
                    f32::INFINITY
                };

                let bounds_size = vec2(bounds_width, bounds_height);
                let max_safe_scale = max_scale_within_bounds(bounds_size, &proj, window_size);

                if scale_constrained.x {
                    proj.scale = proj.scale.min(max_safe_scale.x);
                }

                if scale_constrained.y {
                    proj.scale = proj.scale.min(max_safe_scale.y);
                }
            }

            // Move the camera position to normalize the projection window
            if let (Some(mouse_normalized_screen_pos), true) =
                (mouse_normalized_screen_pos, cam.zoom_to_cursor)
            {
                let proj_size = proj.area.max / old_scale;
                let mouse_world_pos = pos.translation.truncate()
                    + mouse_normalized_screen_pos * proj_size * old_scale;
                pos.translation = (mouse_world_pos
                    - mouse_normalized_screen_pos * proj_size * proj.scale)
                    .extend(pos.translation.z);

                // As we zoom out, we don't want the viewport to move beyond the provided boundary. If the most recent
                // change to the camera zoom would move cause parts of the window beyond the boundary to be shown, we
                // need to change the camera position to keep the viewport within bounds. The four if statements below
                // provide this behavior for the min and max x and y boundaries.
                let proj_size = proj.area.size();

                let half_of_viewport = proj_size / 2.;

                if let Some(min_x_bound) = cam.min_x {
                    let min_safe_cam_x = min_x_bound + half_of_viewport.x;
                    pos.translation.x = pos.translation.x.max(min_safe_cam_x);
                }
                if let Some(max_x_bound) = cam.max_x {
                    let max_safe_cam_x = max_x_bound - half_of_viewport.x;
                    pos.translation.x = pos.translation.x.min(max_safe_cam_x);
                }
                if let Some(min_y_bound) = cam.min_y {
                    let min_safe_cam_y = min_y_bound + half_of_viewport.y;
                    pos.translation.y = pos.translation.y.max(min_safe_cam_y);
                }
                if let Some(max_y_bound) = cam.max_y {
                    let max_safe_cam_y = max_y_bound - half_of_viewport.y;
                    pos.translation.y = pos.translation.y.min(max_safe_cam_y);
                }
            }
        }
    }
}

/// max_scale_within_bounds is used to find the maximum safe zoom out/projection
/// scale when we have been provided with minimum and maximum x boundaries for
/// the camera.
fn max_scale_within_bounds(
    bounds_size: Vec2,
    proj: &OrthographicProjection,
    window_size: Vec2, //viewport?
) -> Vec2 {
    let mut p = proj.clone();
    p.scale = 1.;
    p.update(window_size.x, window_size.y);
    let base_world_size = p.area.size();
    bounds_size / base_world_size
}

#[cfg(not(feature = "leafwing-input-manager"))]
fn check_mouse_interaction<'a>(
    mouse_buttons: &'a Res<ButtonInput<MouseButton>>,
) -> impl Fn(&'a MouseButton) -> bool {
    |btn| mouse_buttons.pressed(*btn) && !mouse_buttons.just_pressed(*btn)
}

#[cfg(feature = "leafwing-input-manager")]
fn check_leafwing_interaction(query: &Query<&ActionState<PanCamAction>>) -> bool {
    let Ok(action_state) = query.get_single() else {
        return false;
    };
    action_state.pressed(&PanCamAction::Grab) && !action_state.just_pressed(&PanCamAction::Grab)
}

fn camera_movement(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    #[cfg(feature = "leafwing-input-manager")] mut query: Query<(
        &PanCam,
        &PanCam,
        &mut Transform,
        &OrthographicProjection,
    )>,
    #[cfg(not(feature = "leafwing-input-manager"))] mut query: Query<(
        &PanCam,
        &PanCamGrabButtons,
        &mut Transform,
        &OrthographicProjection,
    )>,
    mut last_pos: Local<Option<Vec2>>,
    #[cfg(not(feature = "leafwing-input-manager"))] mouse_buttons: Res<ButtonInput<MouseButton>>,
    #[cfg(feature = "leafwing-input-manager")] action_query: Query<&ActionState<PanCamAction>>,
) {
    if let Ok(window) = primary_window.get_single() {
        let window_size = Vec2::new(window.width(), window.height());

        // Use position instead of MouseMotion, otherwise we don't get acceleration movement
        let current_pos = match window.cursor_position() {
            Some(c) => Vec2::new(c.x, -c.y),
            None => return,
        };
        let delta_device_pixels = current_pos - last_pos.unwrap_or(current_pos);

        for (cam, _inputs, mut transform, projection) in &mut query {
            #[cfg(not(feature = "leafwing-input-manager"))]
            let grabbing = _inputs
                .0
                .iter()
                .any(check_mouse_interaction(&mouse_buttons));
            #[cfg(feature = "leafwing-input-manager")]
            let grabbing = check_leafwing_interaction(&action_query);

            if cam.enabled && grabbing {
                let proj_size = projection.area.size();

                let world_units_per_device_pixel = proj_size / window_size;

                // The proposed new camera position
                let delta_world = delta_device_pixels * world_units_per_device_pixel;
                let mut proposed_cam_transform = transform.translation - delta_world.extend(0.);

                // Check whether the proposed camera movement would be within the provided boundaries, override it if we
                // need to do so to stay within bounds.
                if let Some(min_x_boundary) = cam.min_x {
                    let min_safe_cam_x = min_x_boundary + proj_size.x / 2.;
                    proposed_cam_transform.x = proposed_cam_transform.x.max(min_safe_cam_x);
                }
                if let Some(max_x_boundary) = cam.max_x {
                    let max_safe_cam_x = max_x_boundary - proj_size.x / 2.;
                    proposed_cam_transform.x = proposed_cam_transform.x.min(max_safe_cam_x);
                }
                if let Some(min_y_boundary) = cam.min_y {
                    let min_safe_cam_y = min_y_boundary + proj_size.y / 2.;
                    proposed_cam_transform.y = proposed_cam_transform.y.max(min_safe_cam_y);
                }
                if let Some(max_y_boundary) = cam.max_y {
                    let max_safe_cam_y = max_y_boundary - proj_size.y / 2.;
                    proposed_cam_transform.y = proposed_cam_transform.y.min(max_safe_cam_y);
                }

                transform.translation = proposed_cam_transform;
            }
        }
        *last_pos = Some(current_pos);
    }
}

/// Group of leafwing input actions for the `PanCam` component
/// This is only available when the `leafwing-input-manager` feature is enabled
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
#[cfg(feature = "leafwing-input-manager")]
pub enum PanCamAction {
    /// Action to grab the camera
    Grab,
    /// Action to zoom in and out
    Zoom,
}

#[cfg(not(feature = "leafwing-input-manager"))]
#[derive(Component)]
/// Component that adds panning camera controls to an orthographic camera
pub struct PanCamGrabButtons(pub Vec<MouseButton>);

/// Bundle for adding panning camera controls to an orthographic camera
/// This bundle adds both the `PanCam` component and the necessary input handling
#[derive(Bundle)]
pub struct PanCamBundle {
    /// The panning camera component
    pub pan_cam: PanCam,
    #[cfg(feature = "leafwing-input-manager")]
    /// The input manager bundle for the panning camera
    pub inputs: InputManagerBundle<PanCamAction>,
    #[cfg(not(feature = "leafwing-input-manager"))]
    /// The mouse buttons that can be used to grab the camera
    pub inputs: PanCamGrabButtons,
}

impl Default for PanCamBundle {
    fn default() -> Self {
        Self {
            pan_cam: PanCam::default(),
            #[cfg(feature = "leafwing-input-manager")]
            inputs: InputManagerBundle::<PanCamAction> {
                action_state: ActionState::default(),
                input_map: InputMap::default()
                    .insert_multiple([
                        (PanCamAction::Grab, MouseButton::Left),
                        (PanCamAction::Grab, MouseButton::Middle),
                        (PanCamAction::Grab, MouseButton::Right),
                    ])
                    .insert(PanCamAction::Zoom, SingleAxis::mouse_wheel_y())
                    .build(),
            },
            #[cfg(not(feature = "leafwing-input-manager"))]
            inputs: PanCamGrabButtons(vec![
                MouseButton::Left,
                MouseButton::Right,
                MouseButton::Middle,
            ]),
        }
    }
}

/// A component that adds panning camera controls to an orthographic camera
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PanCam {
    /// Whether camera currently responds to user input
    pub enabled: bool,
    /// When true, zooming the camera will center on the mouse cursor
    ///
    /// When false, the camera will stay in place, zooming towards the
    /// middle of the screen
    pub zoom_to_cursor: bool,
    /// The minimum scale for the camera
    ///
    /// The orthographic projection's scale will be clamped at this value when zooming in
    pub min_scale: f32,
    /// The maximum scale for the camera
    ///
    /// If present, the orthographic projection's scale will be clamped at
    /// this value when zooming out.
    pub max_scale: Option<f32>,
    /// The minimum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub min_x: Option<f32>,
    /// The maximum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub max_x: Option<f32>,
    /// The minimum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub min_y: Option<f32>,
    /// The maximum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub max_y: Option<f32>,
}

impl Default for PanCam {
    fn default() -> Self {
        Self {
            enabled: true,
            zoom_to_cursor: true,
            min_scale: 0.00001,
            max_scale: None,
            min_x: None,
            max_x: None,
            min_y: None,
            max_y: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::INFINITY;

    use super::*;

    /// Simple mock function to construct a square projection from a window size
    fn mock_proj(window_size: Vec2) -> OrthographicProjection {
        let mut proj = Camera2dBundle::default().projection;
        proj.update(window_size.x, window_size.y);
        proj
    }

    #[test]
    fn bounds_matching_window_width_have_max_scale_1() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(100., INFINITY), &proj, window_size).x,
            1.
        );
    }

    // boundaries are 1/2 the size of the projection window
    #[test]
    fn bounds_half_of_window_width_have_half_max_scale() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(50., INFINITY), &proj, window_size).x,
            0.5
        );
    }

    // boundaries are 2x the size of the projection window
    #[test]
    fn bounds_twice_of_window_width_have_max_scale_2() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(200., INFINITY), &proj, window_size).x,
            2.
        );
    }

    #[test]
    fn bounds_matching_window_height_have_max_scale_1() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(INFINITY, 100.), &proj, window_size).y,
            1.
        );
    }

    // boundaries are 1/2 the size of the projection window
    #[test]
    fn bounds_half_of_window_height_have_half_max_scale() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(INFINITY, 50.), &proj, window_size).y,
            0.5
        );
    }

    // boundaries are 2x the size of the projection window
    #[test]
    fn bounds_twice_of_window_height_have_max_scale_2() {
        let window_size = vec2(100., 100.);
        let proj = mock_proj(window_size);
        assert_eq!(
            max_scale_within_bounds(vec2(INFINITY, 200.), &proj, window_size).y,
            2.
        );
    }
}
