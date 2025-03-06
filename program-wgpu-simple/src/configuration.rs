use serde::{Deserialize, Serialize};
use shared::*;

pub const DEFAULT_WIDTH: u32 = 800;
pub const DEFAULT_HEIGHT: u32 = 480;
pub const DEFAULT_DISTANCE: u32 = 170;

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
    pub playing: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Globals {
    pub fixed_delta_time: f32,
    pub time_scale: f32,
    pub compute_steps_per_render: u32,
    pub click_mode: ClickMode,
    pub brush_size: f32,
}

pub const GLOBALS: Globals = Globals {
    fixed_delta_time: 1.0 / 120.0,
    time_scale: 0.2,
    compute_steps_per_render: 1,
    click_mode: ClickMode::PaintTrail(0),
    brush_size: 5.0,
};

#[cfg_attr(not(target_arch = "spirv"), derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub struct AgentStatsAll {
    pub name: String,
    pub spawn_mode: SpawnMode,
    pub num_agents: usize,
    pub shader_stats: AgentStats,
}

pub fn create_agent_stats_all() -> [AgentStatsAll; NUM_AGENT_TYPES] {
    [
        AgentStatsAll {
            name: "Red".to_string(),
            spawn_mode: SpawnMode::CircumferenceFacingInward { distance: 170 },
            num_agents: 0000,
            shader_stats: AgentStats {
                velocity: 65.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 5.0,
                interaction_channels: [
                    TrailInteraction {
                        attraction: 0.2,
                        addition: 1.0 / 5.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 0.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 0.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 0.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                ],
            },
        },
        AgentStatsAll {
            name: "Green".to_string(),
            spawn_mode: SpawnMode::CircumferenceFacingInward { distance: 180 },
            num_agents: 4000,
            shader_stats: AgentStats {
                velocity: 65.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 5.0,
                interaction_channels: [
                    TrailInteraction {
                        attraction: -1.0,
                        addition: 0.0,
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        addition: 1.0 / 5.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 0.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 0.0,
                        addition: 0.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                ],
            },
        },
        AgentStatsAll {
            name: "Blue".to_string(),
            // spawn_mode: SpawnMode::CircumferenceFacingClockwise { distance: 170 },
            spawn_mode: SpawnMode::CircumferenceFacingInward { distance: 190 },
            // spawn_mode: SpawnMode::BoxFacingRandom {
            //     spawn_box: SpawnBox {
            //         left: 400,
            //         top: 225,
            //         box_width: 150,
            //         box_height: 150,
            //     }
            // },
            num_agents: 4000,
            shader_stats: AgentStats {
                velocity: 65.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 5.0,
                interaction_channels: [
                    TrailInteraction {
                        attraction: 0.0,
                        addition: 0.0,
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: -1.0,
                        addition: 0.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        addition: 1.0 / 5.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 0.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                ],
            },
        },
        AgentStatsAll {
            name: "Gray".to_string(),
            // spawn_mode: SpawnMode::CircumferenceFacingClockwise { distance: 170 },
            spawn_mode: SpawnMode::CircumferenceFacingInward { distance: 210 },
            // spawn_mode: SpawnMode::BoxFacingRandom {
            //     spawn_box: SpawnBox {
            //         left: 250,
            //         top: 225,
            //         box_width: 150,
            //         box_height: 150,
            //     }
            // },
            num_agents: 4000,
            shader_stats: AgentStats {
                velocity: 65.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 5.0,
                interaction_channels: [
                    TrailInteraction {
                        attraction: 1.0,
                        addition: 0.0,
                        conversion_enabled: 1,
                        conversion_threshold: 0.8,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 0.0,
                        addition: 0.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: -1.0,
                        addition: 0.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction {
                        attraction: 0.2,
                        addition: 1.0 / 5.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                ],
            },
        },
    ]
}

pub const TRAIL_NAMES: [&'static str; NUM_TRAIL_STATS] = [
    "Red",
    "Green",
    "Blue",
    "Gray",
];

pub const TRAIL_STATS: [TrailStats; NUM_TRAIL_STATS] = [
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
    },
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
    },
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
    },
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 480.0,
    },
];
