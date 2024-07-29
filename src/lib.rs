#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::{
        bounding::{Aabb2d, BoundingVolume},
        vec2, Rect,
    },
    prelude::*,
    render::camera::CameraProjection,
    window::PrimaryWindow,
};
use std::ops::RangeInclusive;

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
            (do_camera_movement, do_camera_zoom).in_set(PanCamSystemSet),
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

fn do_camera_zoom(
    mut query: Query<(&PanCam, &mut OrthographicProjection, &mut Transform)>,
    scroll_events: EventReader<MouseWheel>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    const ZOOM_SENSITIVITY: f32 = 0.001;

    let scroll_offset = scroll_offset_from_events(scroll_events);
    if scroll_offset == 0. {
        return;
    }

    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let window_size = window.size();

    let cursor_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2. - Vec2::ONE)
        .map(|p| vec2(p.x, -p.y));

    for (cam, mut proj, mut pos) in &mut query {
        if !cam.enabled {
            continue;
        }

        let old_scale = proj.scale;
        proj.scale *= 1. - scroll_offset * ZOOM_SENSITIVITY;

        constrain_proj_scale(
            &mut proj,
            cam.rect().size(),
            &cam.scale_range(),
            window_size,
        );

        // Move the camera position to normalize the projection window
        let (Some(cursor_normalized_screen_pos), true) =
            (cursor_normalized_screen_pos, cam.zoom_to_cursor)
        else {
            continue;
        };

        let proj_size = proj.area.max / old_scale;
        let cursor_world_pos =
            pos.translation.truncate() + cursor_normalized_screen_pos * proj_size * old_scale;
        let proposed_cam_pos =
            cursor_world_pos - cursor_normalized_screen_pos * proj_size * proj.scale;

        // As we zoom out, we don't want the viewport to move beyond the provided
        // boundary. If the most recent change to the camera zoom would move cause
        // parts of the window beyond the boundary to be shown, we need to change the
        // camera position to keep the viewport within bounds.
        let aabb = cam.aabb().shrink(proj.area.size() / 2.);
        pos.translation = proposed_cam_pos
            .clamp(aabb.min, aabb.max)
            .extend(pos.translation.z);
    }
}

/// Consumes `MouseWheel` event reader and calculates a single scalar,
/// representing positive or negative scroll offset.
fn scroll_offset_from_events(mut scroll_events: EventReader<MouseWheel>) -> f32 {
    let pixels_per_line = 100.; // Maybe make configurable?
    scroll_events
        .read()
        .map(|ev| match ev.unit {
            MouseScrollUnit::Pixel => ev.y,
            MouseScrollUnit::Line => ev.y * pixels_per_line,
        })
        .sum::<f32>()
}

/// `max_scale_within_bounds` is used to find the maximum safe zoom out/projection
/// scale when we have been provided with minimum and maximum x boundaries for
/// the camera.
fn max_scale_within_bounds(
    bounded_area_size: Vec2,
    proj: &OrthographicProjection,
    window_size: Vec2, //viewport?
) -> Vec2 {
    let mut proj = proj.clone();
    proj.scale = 1.;
    proj.update(window_size.x, window_size.y);
    let base_world_size = proj.area.size();
    bounded_area_size / base_world_size
}

/// Makes sure that the camera projection scale stays in the provided bounds
/// and range.
fn constrain_proj_scale(
    proj: &mut OrthographicProjection,
    bounded_area_size: Vec2,
    scale_range: &RangeInclusive<f32>,
    window_size: Vec2,
) {
    proj.scale = proj.scale.clamp(*scale_range.start(), *scale_range.end());

    // If there is both a min and max boundary, that limits how far we can zoom.
    // Make sure we don't exceed that
    if bounded_area_size.x.is_finite() || bounded_area_size.y.is_finite() {
        let max_safe_scale = max_scale_within_bounds(bounded_area_size, &proj, window_size);
        proj.scale = proj.scale.min(max_safe_scale.x).min(max_safe_scale.y);
    }
}

fn do_camera_movement(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&PanCam, &mut Transform, &OrthographicProjection)>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };
    let window_size = window.size();

    // Use position instead of MouseMotion, otherwise we don't get acceleration
    // movement
    let current_pos = match window.cursor_position() {
        Some(c) => vec2(c.x, -c.y),
        None => return,
    };
    let delta_device_pixels = current_pos - last_pos.unwrap_or(current_pos);

    for (cam, mut transform, projection) in &mut query {
        if !cam.enabled
            || !cam
                .grab_buttons
                .iter()
                .any(|btn| mouse_buttons.pressed(*btn) && !mouse_buttons.just_pressed(*btn))
        {
            continue;
        }

        let proj_area_size = projection.area.size();
        let world_units_per_device_pixel = proj_area_size / window_size;

        // The proposed new camera position
        let delta_world = delta_device_pixels * world_units_per_device_pixel;
        let aabb = cam.aabb().shrink(proj_area_size / 2.);
        let proposed_cam_pos =
            (transform.translation.truncate() - delta_world).clamp(aabb.min, aabb.max);

        transform.translation = proposed_cam_pos.extend(transform.translation.z);
    }
    *last_pos = Some(current_pos);
}

/// A component that adds panning camera controls to an orthographic camera
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PanCam {
    /// The mouse buttons that will be used to drag and pan the camera
    pub grab_buttons: Vec<MouseButton>,
    /// Whether camera currently responds to user input
    pub enabled: bool,
    /// When true, zooming the camera will center on the mouse cursor
    ///
    /// When false, the camera will stay in place, zooming towards the
    /// middle of the screen
    pub zoom_to_cursor: bool,
    /// The minimum scale for the camera
    ///
    /// The orthographic projection's scale will be clamped at this value when
    /// zooming in
    pub min_scale: f32,
    /// The maximum scale for the camera
    ///
    /// If present, the orthographic projection's scale will be clamped at
    /// this value when zooming out.
    pub max_scale: Option<f32>,
    /// The minimum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary
    /// both when dragging the window, and zooming out.
    pub min_x: Option<f32>,
    /// The maximum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary
    /// both when dragging the window, and zooming out.
    pub max_x: Option<f32>,
    /// The minimum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary
    /// both when dragging the window, and zooming out.
    pub min_y: Option<f32>,
    /// The maximum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary
    /// both when dragging the window, and zooming out.
    pub max_y: Option<f32>,
}

impl PanCam {
    /// Returns (min, max) bound tuple. If some bounds were not provided, either
    /// `f32::INFINITY` or `f32::NEG_INFINITY` will be used.
    fn bounds(&self) -> (Vec2, Vec2) {
        let min = vec2(
            self.min_x.unwrap_or(f32::NEG_INFINITY),
            self.min_y.unwrap_or(f32::NEG_INFINITY),
        );
        let max = vec2(
            self.max_x.unwrap_or(f32::INFINITY),
            self.max_y.unwrap_or(f32::INFINITY),
        );
        (min, max)
    }

    /// Returns the bounding `Rect`. If some bounds were not provided, either
    /// `f32::INFINITY` or `f32::NEG_INFINITY` will be used.
    fn rect(&self) -> Rect {
        let (min, max) = self.bounds();
        Rect { min, max }
    }

    /// Returns the bounding `Aabb2d`. If some bounds were not provided, either
    /// `f32::INFINITY` or `f32::NEG_INFINITY` will be used.
    ///
    /// Since bevy doesn't provide a `shrink` method on a `Rect` yet, we have to
    /// additionally return the `Aabb2d` type .
    fn aabb(&self) -> Aabb2d {
        let (min, max) = self.bounds();
        Aabb2d { min, max }
    }

    /// Returns the scale inclusive range. If upper bound was not provided,
    /// `f32::INFINITY` will be used.
    fn scale_range(&self) -> RangeInclusive<f32> {
        self.min_scale..=self.max_scale.unwrap_or(f32::INFINITY)
    }
}

impl Default for PanCam {
    fn default() -> Self {
        Self {
            grab_buttons: vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle],
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

    use bevy::prelude::OrthographicProjection;

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
