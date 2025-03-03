use egui_winit::State;
use crate::configuration::Globals;
use shared::{AgentStatsAll, NUM_AGENT_TYPES, NUM_TRAIL_STATS, TrailStats};

pub struct ConfigurationValues {
    pub globals: Globals,
    pub agent_stats: [AgentStatsAll; NUM_AGENT_TYPES],
    pub trail_stats: [TrailStats; NUM_TRAIL_STATS],
    pub scale_factor: f32,
    pub show_menu: bool,
}

pub fn render_configuration_menu(state: &State, configuration: &mut ConfigurationValues) {
    egui::Window::new("Configuration")
        .resizable(true)
        .vscroll(true)
        .default_open(false)
        .show(state.egui_ctx(), |ui| {
            ui.label("Label!");

            if ui.button("Button!").clicked() {
                println!("boom!")
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Pixels per point: {}",
                    state.egui_ctx().pixels_per_point()
                ));
                if ui.button("-").clicked() {
                    configuration.scale_factor = (configuration.scale_factor - 0.1).max(0.3);
                }
                if ui.button("+").clicked() {
                    configuration.scale_factor = (configuration.scale_factor + 0.1).min(3.0);
                }
            });
        });
}
