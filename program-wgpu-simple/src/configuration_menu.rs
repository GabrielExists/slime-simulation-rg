use egui::Slider;
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
    if configuration.show_menu {
        egui::Window::new("Configuration")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(state.egui_ctx(), |ui| {
                if ui.button("Hide").clicked() {
                    configuration.show_menu = !configuration.show_menu;
                }
                for (agent_index, agent_stats) in configuration.agent_stats.iter_mut().enumerate() {
                    ui.collapsing(format!("Agent {}", agent_index), |ui| {
                        ui.add(Slider::new(&mut agent_stats.shader_stats.avoidance_threshold, 0.0..=20.0)
                            .text("Avoidance Threshold"));
                    });
                }
                for (trail_index, trail_stats) in configuration.trail_stats.iter_mut().enumerate() {
                    ui.collapsing(format!("Trail {}", trail_index), |ui| {
                        ui.add(Slider::new(&mut trail_stats.diffusion_speed, 0.0..=1000.0)
                            .text("Diffusion Speed"));
                    });
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
}
