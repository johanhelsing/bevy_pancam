use bevy::prelude::*;
use bevy_egui_0_39::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, ScrollArea},
};
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::random;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin, EguiPlugin::default()))
        .add_systems(EguiPrimaryContextPass, egui_ui)
        .add_systems(Startup, setup)
        .run();
}

fn egui_ui(mut contexts: EguiContexts) -> Result {
    egui::Window::new("Scroll me")
        .resizable(false)
        .show(contexts.ctx_mut()?, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(100.);
                ui.color_edit_button_rgb(&mut [0., 0., 0.]);
                ui.add(egui::Slider::new(&mut 0.0, 0.0..=1.0).step_by(0.001));
                ui.checkbox(&mut true, "Test");
                ui.vertical(|ui| {
                    for i in 0..50 {
                        ui.label(format!("list entry number {i}"));
                    }
                });
            });
        });
    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, PanCam::default()));

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
