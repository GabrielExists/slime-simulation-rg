use crate::configuration::ConfigurationValues;
use winit::dpi::PhysicalSize;
use crate::configuration::TRAIL_NAMES;
use egui::{Slider, Ui};
use egui::ComboBox;
use egui_winit::State;
use crate::configuration::DEFAULT_DISTANCE;
use shared::{ClickMode, NUM_TRAIL_STATS, SpawnBox, SpawnMode};

pub fn render_configuration_menu(state: &State, screen_size: PhysicalSize<u32>, configuration: &mut ConfigurationValues) {
    configuration.shader_config_changed = false;
    if configuration.show_menu {
        egui::Window::new("Configuration")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(state.egui_ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Hide").clicked() {
                        configuration.show_menu = !configuration.show_menu;
                    }
                    if ui.button(if configuration.playing { "Pause" } else { "Resume" }).clicked() {
                        configuration.playing = !configuration.playing;
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Respawn").clicked() {
                        configuration.respawn = true;
                    }
                    if ui.button("Reset trails").clicked() {
                        configuration.reset_trails = true;
                    }
                });
                if ui.button("Respawn and reset trails").clicked() {
                    configuration.respawn = true;
                    configuration.reset_trails = true;
                }
                for agent_stats in configuration.agent_stats.iter_mut() {
                    let spawn = &mut agent_stats.spawn_mode;
                    ui.collapsing(format!("Agent {}", agent_stats.name), |ui| {
                        ui.add(Slider::new(&mut agent_stats.shader_stats.velocity, 5.0..=100.0)
                            .text("Velocity"));
                        ui.add(Slider::new(&mut agent_stats.shader_stats.pixel_addition, 0.01..=1.0)
                            .text("Pixel addition divisor").logarithmic(true));
                        ui.add(Slider::new(&mut agent_stats.shader_stats.turn_speed, 5.0..=100.0)
                            .text("Turn speed"));
                        ui.add(Slider::new(&mut agent_stats.shader_stats.turn_speed_avoidance, 5.0..=100.0)
                            .text("Turn speed avoidance"));
                        ui.add(Slider::new(&mut agent_stats.shader_stats.avoidance_threshold, 0.0..=20.0)
                            .text("Avoidance threshold"));
                        ui.add(Slider::new(&mut agent_stats.shader_stats.sensor_angle_spacing, 10.0..=170.0)
                            .text("Sensor angle spacing (degrees)"));
                        ui.add(Slider::new(&mut agent_stats.shader_stats.sensor_offset, 3.0..=30.0)
                            .text("Sensor offset"));
                        ui.add(Slider::new(&mut agent_stats.shader_stats.attraction_channel_one, -1.0..=1.0)
                            .text(format!("Attraction channel {}", TRAIL_NAMES[0])));
                        ui.add(Slider::new(&mut agent_stats.shader_stats.attraction_channel_two, -1.0..=1.0)
                            .text(format!("Attraction channel {}", TRAIL_NAMES[1])));
                        ui.separator();
                        ui.label("Applies on reset:");
                        // ui.add(ComboBox::new(&mut agent_stats.spawn_mode, "Spawn mode"));
                        ui.add(Slider::new(&mut agent_stats.num_agents, 5..=1000000)
                            .text("Num agents").logarithmic(true));
                        ComboBox::from_label("Spawn mode")
                            .selected_text(format!("{}", spawn))
                            .show_ui(ui, |ui| {
                                selectable_value_pred(ui, spawn, |mode| matches!(mode, SpawnMode::EvenlyDistributed), SpawnMode::EvenlyDistributed {});
                                selectable_value_pred(ui, spawn, |mode| matches!(mode, SpawnMode::CenterFacingOutward {}), SpawnMode::CenterFacingOutward {});
                                selectable_value_pred(ui, spawn, |mode| matches!(mode, SpawnMode::PointFacingOutward {..}), SpawnMode::PointFacingOutward { x: 100, y: 100 });
                                selectable_value_pred(ui, spawn, |mode| matches!(mode, SpawnMode::CircleFacingInward {..}), SpawnMode::CircleFacingInward {
                                    max_distance: spawn.distance().unwrap_or(DEFAULT_DISTANCE)
                                });
                                selectable_value_pred(ui, spawn, |mode| matches!(mode, SpawnMode::CircumferenceFacingInward {..}), SpawnMode::CircumferenceFacingInward {
                                    distance: spawn.distance().unwrap_or(DEFAULT_DISTANCE)
                                });
                                selectable_value_pred(ui, spawn, |mode| matches!(mode, SpawnMode::CircumferenceFacingOutward {..}), SpawnMode::CircumferenceFacingOutward {
                                    distance: spawn.distance().unwrap_or(DEFAULT_DISTANCE)
                                });
                                selectable_value_pred(ui, spawn, |mode| matches!(mode, SpawnMode::CircumferenceFacingRandom {..}), SpawnMode::CircumferenceFacingRandom {
                                    distance: spawn.distance().unwrap_or(DEFAULT_DISTANCE)
                                });
                                selectable_value_pred(ui, spawn, |mode| matches!(mode, SpawnMode::CircumferenceFacingClockwise {..}), SpawnMode::CircumferenceFacingClockwise {
                                    distance: spawn.distance().unwrap_or(DEFAULT_DISTANCE)
                                });
                                selectable_value_pred(ui, spawn, |mode| matches!(mode, SpawnMode::BoxFacingRandom {..}), SpawnMode::BoxFacingRandom {
                                    spawn_box: spawn.spawn_box().unwrap_or(SpawnBox::default())
                                });
                            });
                        let width = screen_size.width;
                        let height = screen_size.height;
                        let diagonal_max_radius = (
                            (width as f32 / 2.0).powi(2) + (height as f32 / 2.0).powi(2)
                        ).sqrt() as u32;
                        match spawn {
                            SpawnMode::EvenlyDistributed => {}
                            SpawnMode::CenterFacingOutward => {}
                            SpawnMode::PointFacingOutward { x, y } => {
                                ui.add(Slider::new(x, 0..=width)
                                    .text("X"));
                                ui.add(Slider::new(y, 0..=height)
                                    .text("Y"));
                            }
                            SpawnMode::CircleFacingInward { max_distance } => {
                                ui.add(Slider::new(max_distance, 0..=diagonal_max_radius)
                                    .text("Max distance"));
                            }
                            SpawnMode::CircumferenceFacingInward { distance } => {
                                ui.add(Slider::new(distance, 0..=diagonal_max_radius)
                                    .text("Distance"));
                            }
                            SpawnMode::CircumferenceFacingOutward { distance } => {
                                ui.add(Slider::new(distance, 0..=diagonal_max_radius)
                                    .text("Distance"));
                            }
                            SpawnMode::CircumferenceFacingRandom { distance } => {
                                ui.add(Slider::new(distance, 0..=diagonal_max_radius)
                                    .text("Distance"));
                            }
                            SpawnMode::CircumferenceFacingClockwise { distance } => {
                                ui.add(Slider::new(distance, 0..=diagonal_max_radius)
                                    .text("Distance"));
                            }
                            SpawnMode::BoxFacingRandom { spawn_box: SpawnBox { left, top, box_width, box_height } } => {
                                ui.add(Slider::new(left, 0..=width)
                                    .text("Left"));
                                ui.add(Slider::new(top, 0..=height)
                                    .text("Top"));
                                ui.add(Slider::new(box_width, 0..=width - *left)
                                    .text("Width"));
                                ui.add(Slider::new(box_height, 0..=height - *top)
                                    .text("Height"));
                            }
                        }
                    });
                }
                for (trail_index, trail_stats) in configuration.trail_stats.iter_mut().enumerate() {
                    ui.collapsing(format!("Trail {}", TRAIL_NAMES[trail_index]), |ui| {
                        ui.add(Slider::new(&mut trail_stats.evaporation_speed, 0.0..=1000.0)
                            .text("Evaporation speed"));
                        ui.add(Slider::new(&mut trail_stats.diffusion_speed, 0.0..=1000.0)
                            .text("Diffusion speed"));
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
                let click_mode = &mut configuration.globals.click_mode;
                ComboBox::from_label("Click mode")
                    .selected_text(format!("{}", click_mode))
                    .show_ui(ui, |ui| {
                        selectable_value_pred(ui, click_mode, |mode| matches!(mode, ClickMode::Disabled), ClickMode::Disabled);
                        selectable_value_pred(ui, click_mode, |mode| matches!(mode, ClickMode::ShowMenu), ClickMode::ShowMenu);
                        selectable_value_pred(ui, click_mode, |mode| matches!(mode, ClickMode::PaintTrail(_)), ClickMode::PaintTrail(0));
                        selectable_value_pred(ui, click_mode, |mode| matches!(mode, ClickMode::ResetTrail(_)), ClickMode::ResetTrail(0));
                        selectable_value_pred(ui, click_mode, |mode| matches!(mode, ClickMode::ResetAllTrails), ClickMode::ResetAllTrails);
                    });
                match click_mode {
                    ClickMode::PaintTrail(trail_index) => {
                        ui.add(Slider::new(trail_index, 0..=NUM_TRAIL_STATS as u32 - 1)
                            .text("Trail index"));
                    }
                    ClickMode::ResetTrail(trail_index) => {
                        ui.add(Slider::new(trail_index, 0..=NUM_TRAIL_STATS as u32 - 1)
                            .text("Trail index"));
                    }
                    _ => {}
                }
            });
    }
}

pub fn selectable_value_pred<Value: std::fmt::Display, F>(
    ui: &mut Ui,
    current_value: &mut Value,
    predicate: F,
    selected_value: Value,
) -> egui::Response
    where F: Fn(&Value) -> bool {
    let selected = predicate(current_value);
    let mut response = ui.selectable_label(selected, selected_value.to_string());
    if response.clicked() && !selected {
        *current_value = selected_value;
        response.mark_changed();
    }
    response
}
