use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::camera::OrthographicProjection,
};

#[cfg(feature = "bevy-inspector-egui")]
use bevy_inspector_egui::InspectableRegistry;

/// Plugin that adds the necessary systems for `PanCam` components to work
#[derive(Default)]
pub struct PanCamPlugin;

impl Plugin for PanCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_movement).add_system(camera_zoom);

        #[cfg(feature = "bevy-inspector-egui")]
        app.add_plugin(InspectablePlugin);
    }
}

fn camera_zoom(
    mut query: Query<(&PanCam, &mut OrthographicProjection, &mut Transform)>,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Res<Windows>,
    #[cfg(feature = "bevy_egui")] egui_ctx: Option<ResMut<bevy_egui::EguiContext>>,
) {
    #[cfg(feature = "bevy_egui")]
    if let Some(mut egui_ctx) = egui_ctx {
        if egui_ctx.ctx_mut().wants_pointer_input() || egui_ctx.ctx_mut().wants_keyboard_input() {
            return;
        }
    }
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
    let mouse_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2. - Vec2::ONE);

    for (cam, mut proj, mut pos) in &mut query {
        if cam.enabled {
            let old_scale = proj.scale;
            proj.scale = (proj.scale * (1. + -scroll * 0.001)).max(cam.min_scale);

            // Apply max scale constraint
            if let Some(max_scale) = cam.max_scale {
                proj.scale = proj.scale.min(max_scale);
            }

            // If there is both a min and max x boundary, that limits how far we can zoom. Make sure we don't exceed that
            if let (Some(min_x_bound), Some(max_x_bound)) = (cam.min_x, cam.max_x) {
                let max_safe_scale = max_scale_within_x_bounds(min_x_bound, max_x_bound, &proj);
                proj.scale = proj.scale.min(max_safe_scale);
            }
            // If there is both a min and max y boundary, that limits how far we can zoom. Make sure we don't exceed that
            if let (Some(min_y_bound), Some(max_y_bound)) = (cam.min_y, cam.max_y) {
                let max_safe_scale = max_scale_within_y_bounds(min_y_bound, max_y_bound, &proj);
                proj.scale = proj.scale.min(max_safe_scale);
            }

            // Move the camera position to normalize the projection window
            if let (Some(mouse_normalized_screen_pos), true) =
                (mouse_normalized_screen_pos, cam.zoom_to_cursor)
            {
                let proj_size = Vec2::new(proj.right, proj.top);
                let mouse_world_pos = pos.translation.truncate()
                    + mouse_normalized_screen_pos * proj_size * old_scale;
                pos.translation = (mouse_world_pos
                    - mouse_normalized_screen_pos * proj_size * proj.scale)
                    .extend(pos.translation.z);

                // As we zoom out, we don't want the viewport to move beyond the provided boundary. If the most recent
                // change to the camera zoom would move cause parts of the window beyond the boundary to be shown, we
                // need to change the camera position to keep the viewport within bounds. The four if statements below
                // provide this behavior for the min and max x and y boundaries.
                let scaling = Vec2::new(
                    window.width() / (proj.right - proj.left),
                    window.height() / (proj.top - proj.bottom),
                ) * proj.scale;
                if let Some(min_x_bound) = cam.min_x {
                    let half_of_viewport = (window.width() / 2.) * scaling.x;
                    let min_safe_cam_x = min_x_bound + half_of_viewport;
                    pos.translation.x = pos.translation.x.max(min_safe_cam_x);
                }
                if let Some(max_x_bound) = cam.max_x {
                    let half_of_viewport = (window.width() / 2.) * scaling.x;
                    let max_safe_cam_x = max_x_bound - half_of_viewport;
                    pos.translation.x = pos.translation.x.min(max_safe_cam_x);
                }
                if let Some(min_y_bound) = cam.min_y {
                    let half_of_viewport = (window.height() / 2.) * scaling.y;
                    let min_safe_cam_y = min_y_bound + half_of_viewport;
                    pos.translation.y = pos.translation.y.max(min_safe_cam_y);
                }
                if let Some(max_y_bound) = cam.max_y {
                    let half_of_viewport = (window.height() / 2.) * scaling.y;
                    let max_safe_cam_y = max_y_bound - half_of_viewport;
                    pos.translation.y = pos.translation.y.min(max_safe_cam_y);
                }
            }
        }
    }
}

/// max_scale_within_x_bounds is used to find the maximum safe zoom out/projection scale when we have been provided with
/// minimum and maximum x boundaries for the camera.
fn max_scale_within_x_bounds(
    min_x_bound: f32,
    max_x_bound: f32,
    proj: &OrthographicProjection,
) -> f32 {
    let bounds_width = max_x_bound - min_x_bound;

    // projection width in world space:
    // let proj_width = (proj.right - proj.left) * proj.scale;

    // we're at the boundary when proj_width == bounds_width
    // that means (proj.right - proj.left) * scale == bounds_width

    // if we solve for scale, we get:
    bounds_width / (proj.right - proj.left)
}

/// max_scale_within_y_bounds is used to find the maximum safe zoom out/projection scale when we have been provided with
/// minimum and maximum y boundaries for the camera. It behaves identically to max_scale_within_x_bounds but uses the
/// height of the window and projection instead of their width.
fn max_scale_within_y_bounds(
    min_y_bound: f32,
    max_y_bound: f32,
    proj: &OrthographicProjection,
) -> f32 {
    let bounds_height = max_y_bound - min_y_bound;

    // projection height in world space:
    // let proj_height = (proj.top - proj.bottom) * proj.scale;

    // we're at the boundary when proj_height == bounds_height
    // that means (proj.top - proj.bottom) * scale == bounds_height

    // if we solve for scale, we get:
    bounds_height / (proj.top - proj.bottom)
}

fn camera_movement(
    windows: Res<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut query: Query<(&PanCam, &mut Transform, &OrthographicProjection)>,
    mut last_pos: Local<Option<Vec2>>,
    #[cfg(feature = "bevy_egui")] egui_ctx: Option<ResMut<bevy_egui::EguiContext>>,
) {
    #[cfg(feature = "bevy_egui")]
    if let Some(mut egui_ctx) = egui_ctx {
        if egui_ctx.ctx_mut().wants_pointer_input() || egui_ctx.ctx_mut().wants_keyboard_input() {
            *last_pos = None;
            return;
        }
    }

    let window = windows.get_primary().unwrap();

    // Use position instead of MouseMotion, otherwise we don't get acceleration movement
    let current_pos = match window.cursor_position() {
        Some(current_pos) => current_pos,
        None => return,
    };
    let delta = current_pos - last_pos.unwrap_or(current_pos);

    for (cam, mut transform, projection) in &mut query {
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

            // The proposed new camera position
            let mut proposed_cam_transform = transform.translation - (delta * scaling).extend(0.);

            // Check whether the proposed camera movement would be within the provided boundaries, override it if we
            // need to do so to stay within bounds.
            if let Some(min_x_boundary) = cam.min_x {
                let min_safe_cam_x = min_x_boundary + ((window.width() / 2.) * scaling.x);
                proposed_cam_transform.x = proposed_cam_transform.x.max(min_safe_cam_x);
            }
            if let Some(max_x_boundary) = cam.max_x {
                let max_safe_cam_x = max_x_boundary - ((window.width() / 2.) * scaling.x);
                proposed_cam_transform.x = proposed_cam_transform.x.min(max_safe_cam_x);
            }
            if let Some(min_y_boundary) = cam.min_y {
                let min_safe_cam_y = min_y_boundary + ((window.height() / 2.) * scaling.y);
                proposed_cam_transform.y = proposed_cam_transform.y.max(min_safe_cam_y);
            }
            if let Some(max_y_boundary) = cam.max_y {
                let max_safe_cam_y = max_y_boundary - ((window.height() / 2.) * scaling.y);
                proposed_cam_transform.y = proposed_cam_transform.y.min(max_safe_cam_y);
            }

            transform.translation = proposed_cam_transform;
        }
    }
    *last_pos = Some(current_pos);
}

/// A component that adds panning camera controls to an orthographic camera
#[derive(Component)]
#[cfg_attr(
    feature = "bevy-inspector-egui",
    derive(bevy_inspector_egui::Inspectable)
)]
pub struct PanCam {
    /// The mouse buttons that will be used to drag and pan the camera
    #[cfg_attr(feature = "bevy-inspector-egui", inspectable(ignore))]
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

#[cfg(feature = "bevy-inspector-egui")]
#[derive(bevy_inspector_egui::Inspectable)]
struct InspectablePlugin;

#[cfg(feature = "bevy-inspector-egui")]
impl Plugin for InspectablePlugin {
    fn build(&self, app: &mut App) {
        let mut inspectable_registry = app
            .world
            .get_resource_or_insert_with(InspectableRegistry::default);

        inspectable_registry.register::<PanCam>();
    }
}
