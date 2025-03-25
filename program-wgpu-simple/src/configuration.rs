use serde::{Deserialize, Serialize};
use shared::*;

pub const DEFAULT_WIDTH: u32 = 1280;
pub const DEFAULT_HEIGHT: u32 = 720;
pub const DEFAULT_MAP_WIDTH: u32 = DEFAULT_WIDTH;
pub const DEFAULT_MAP_HEIGHT: u32 = DEFAULT_HEIGHT;
pub const DEFAULT_DISTANCE: u32 = 200;
pub const RESIZE_MAP_WITH_WINDOW: bool = false;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigurationValues {
    pub globals: Globals,
    pub agent_stats: [AgentStatsAll; NUM_AGENT_TYPES],
    pub trail_stats: [TrailStats; NUM_TRAIL_STATS],
    pub shader_config_changed: bool,
    // CPU only fields
    pub scale_factor: f32,
    pub show_menu: bool,
    pub respawn: bool,
    pub reset_trails: bool,
    pub quit: bool,
    pub playing: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Globals {
    pub time_step: f32,
    pub max_frame_rate: f32,
    pub smoothen_after_max_frame_rate: bool,
    pub compute_steps_per_render: u32,
    pub click_mode: ClickMode,
    pub brush_size: f32,
    pub background_color: Color,
    pub map_width: u32,
    pub map_height: u32,
}

pub const GLOBALS: Globals = Globals {
    time_step: 1.0 / 60.0,
    max_frame_rate: 100.0,
    smoothen_after_max_frame_rate: false,
    compute_steps_per_render: 1,
    click_mode: ClickMode::PaintTrail(0),
    brush_size: 7.0,
    background_color: Color::new(0.0048377407, 0.014973952, 0.040314503, 1.0),
    map_width: if RESIZE_MAP_WITH_WINDOW {
        DEFAULT_WIDTH
    } else {
        DEFAULT_MAP_WIDTH
    },
    map_height: if RESIZE_MAP_WITH_WINDOW {
        DEFAULT_HEIGHT
    } else {
        DEFAULT_MAP_HEIGHT
    },
};

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Default)]
pub struct AgentStatsAll {
    pub name: String,
    pub spawn_mode: Vec<SpawnMode>,
    pub num_agents: usize,
    pub shader_stats: AgentStats,
}

pub fn create_agent_stats_all() -> [AgentStatsAll; NUM_AGENT_TYPES] {
    [
        AgentStatsAll {
            name: "Blue".to_string(),
            // spawn_mode: SpawnMode::CircumferenceFacingClockwise { distance: 170 },
            spawn_mode: vec![SpawnMode::CircumferenceFacingOutward { distance: 100 }],
            // spawn_mode: SpawnMode::BoxFacingRandom {
            //     spawn_box: SpawnBox {
            //         left: 400,
            //         top: 225,
            //         box_width: 150,
            //         box_height: 150,
            //     }
            // },
            num_agents: 50000,
            shader_stats: AgentStats {
                velocity: 40.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 30.0,
                interaction_channels: [
                    TrailInteraction {
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 2,
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        addition: 1.0 / 5.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        AgentStatsAll {
            name: "Green".to_string(),
            spawn_mode: vec![SpawnMode::CircumferenceFacingOutward { distance: 100 }],
            num_agents: 50000,
            shader_stats: AgentStats {
                velocity: 40.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 25.0,
                interaction_channels: [
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 1.0 / 5.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: -1.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 3,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        AgentStatsAll {
            name: "BlueInfection".to_string(),
            shader_stats: AgentStats {
                velocity: 60.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 12.0,
                timeout: 1.5,
                timeout_conversion: 4,
                interaction_channels: [
                    TrailInteraction {
                        addition: 1.0 / 5.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 6,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                ],
            },
            ..Default::default()
        },
        AgentStatsAll {
            name: "GreenInfection".to_string(),
            shader_stats: AgentStats {
                velocity: 60.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 12.0,
                timeout: 6.0,
                timeout_conversion: 5,
                interaction_channels: [
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 7,
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        addition: 1.0 / 5.0,
                        ..Default::default()
                    },
                ],
            },
            ..Default::default()
        },
        AgentStatsAll {
            name: "GrayToBlue".to_string(),
            shader_stats: AgentStats {
                velocity: 25.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 40.0,
                interaction_channels: [
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        addition: 1.0 / 5.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        AgentStatsAll {
            name: "GrayToGreen".to_string(),
            shader_stats: AgentStats {
                velocity: 25.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 40.0,
                interaction_channels: [
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 1,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        addition: 1.0 / 5.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 1,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        AgentStatsAll {
            name: "White".to_string(),
            spawn_mode: vec![
                SpawnMode::PointFacingClockwise {
                    x: DEFAULT_MAP_WIDTH / 2,
                    y: 100,
                    distance: 30,
                },
                SpawnMode::PointFacingClockwise {
                    x: DEFAULT_MAP_WIDTH / 2,
                    y: DEFAULT_MAP_HEIGHT / 2,
                    distance: 30,
                },
                SpawnMode::PointFacingClockwise {
                    x: DEFAULT_MAP_WIDTH / 2,
                    y: DEFAULT_MAP_HEIGHT - 100,
                    distance: 30,
                },
            ],
            num_agents: 3000,
            shader_stats: AgentStats {
                velocity: 2.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 15.0,
                interaction_channels: [
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 0.2,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        },
        AgentStatsAll {
            name: "WhiteTemporaryBlue".to_string(),
            shader_stats: AgentStats {
                velocity: 90.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 15.0,
                timeout: 20.0,
                timeout_conversion: 0,
                interaction_channels: [
                    TrailInteraction {
                        attraction: 1.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        addition: 0.2,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                ],
            },
            ..Default::default()
        },
        AgentStatsAll {
            name: "WhiteTemporaryGreen".to_string(),
            shader_stats: AgentStats {
                velocity: 90.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 15.0,
                timeout: 20.0,
                timeout_conversion: 1,
                interaction_channels: [
                    TrailInteraction {
                        attraction: 1.0,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        addition: 0.2,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                ],
            },
            ..Default::default()
        },
        AgentStatsAll {
            name: "PermanentRed".to_string(),
            spawn_mode: vec![SpawnMode::PointFacingClockwise {
                x: 100,
                y: DEFAULT_MAP_HEIGHT / 2,
                distance: 30,
            }],
            num_agents: 8000,
            shader_stats: AgentStats {
                velocity: 2.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 15.0,
                interaction_channels: [
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 0.2,
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        ..Default::default()
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 0.2,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        },
    ]
}

pub const TRAIL_NAMES: [&'static str; NUM_TRAIL_STATS] =
    ["BlueInfection", "Green", "Blue", "Dead", "Cure", "GreenInfection"];

pub const TRAIL_STATS: [TrailStats; NUM_TRAIL_STATS] = [
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
        padding_1: 0.0,
        color_mode: ColorMode::Add.encode(),
        color: Color::new(200.0/255.0, 0.0, 106.0/255.0, 1.0),
    },
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
        padding_1: 0.0,
        color_mode: ColorMode::Add.encode(),
        color: Color::new(0.0, 1.0, 0.0, 1.0),
    },
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
        padding_1: 0.0,
        color_mode: ColorMode::Add.encode(),
        color: Color::new(0.0, 0.0, 1.0, 1.0),
    },
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
        padding_1: 0.0,
        color_mode: ColorMode::Add.encode(),
        color: Color::new(0.05, 0.05, 0.05, 1.0),
    },
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
        padding_1: 0.0,
        color_mode: ColorMode::Add.encode(),
        color: Color::new(1.0, 248.0 / 255.0, 0.00, 1.0),
    },
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
        padding_1: 0.0,
        color_mode: ColorMode::Add.encode(),
        color: Color::new(1.00, 120.0/255.0, 72.0/255.0, 1.0),
    },
];
