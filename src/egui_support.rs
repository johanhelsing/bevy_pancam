use bevy::prelude::*;

use crate::PanCamSystems;

#[derive(Resource, Deref, DerefMut, PartialEq, Eq, Default)]
struct EguiWantsFocus(bool);

pub(crate) struct EguiPanCamPlugin;

impl Plugin for EguiPanCamPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EguiWantsFocus>()
            .add_systems(PostUpdate, check_egui_wants_focus)
            .configure_sets(
                Update,
                PanCamSystems.run_if(resource_equals(EguiWantsFocus(false))),
            );
    }
}

// todo: make run condition when Bevy supports mutable resources in them
fn check_egui_wants_focus(
    #[cfg(feature = "bevy_egui_0_39")] mut contexts: Query<&mut bevy_egui_0_39::EguiContext>,
    mut wants_focus: ResMut<EguiWantsFocus>,
) {
    let mut new_wants_focus = false;

    #[cfg(feature = "bevy_egui_0_39")]
    {
        let ctx = contexts.iter_mut().next();
        if let Some(ctx) = ctx {
            let ctx = ctx.into_inner().get_mut();
            if ctx.wants_pointer_input() || ctx.wants_keyboard_input() {
                new_wants_focus = true;
            }
        }
    }

    wants_focus.set_if_neq(EguiWantsFocus(new_wants_focus));
}
