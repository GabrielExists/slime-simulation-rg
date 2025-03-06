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
    // There are two break points for frame rates
    // Above the maximum, we no longer speed up the simulation,
    // instead we decrease the delta time to make things smoother
    // Between the minimum and maximum, we vary the simulation speed
    // by taking maximum steps more often the faster the hardware is
    // At the minimum, we take one step per frame.
    // It's mostly a reference for setting maximum time step
    // The bigger the difference between update rate min and max, the more the perceived simulation
    // speed will change across hardwares
    pub update_rate_minimum: f32,
    pub update_rate_maximum: f32,
    // The delta time passed to shaders can be smaller than this but never greater
    // This exists because simulations work worse when
    pub maximum_time_step: f32,
    pub compute_steps_per_render: u32,
    pub click_mode: ClickMode,
    pub brush_size: f32,
}

pub const GLOBALS: Globals = Globals {
    minimum_delta_time: 1.0 / 8.0,
    maximum_time_step: 1.0,
    compute_steps_per_render: 1,
    click_mode: ClickMode::PaintTrail(3),
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
            name: "Blue".to_string(),
            spawn_mode: SpawnMode::CircumferenceFacingInward { distance: 170 },
            num_agents: 4000,
            shader_stats: AgentStats {
                velocity: 65.0,
                turn_speed: 80.0,
                turn_speed_avoidance: 30.0,
                avoidance_threshold: 20.0,
                sensor_angle_spacing: 60.0,
                sensor_offset: 5.0,
                interaction_channels: [
                    TrailInteraction::default(),
                    TrailInteraction {
                        attraction: 0.2,
                        addition: 1.0 / 5.0,
                        conversion_enabled: 0,
                        conversion_threshold: 0.0,
                        conversion: 0,
                    },
                    TrailInteraction::default(),
                    TrailInteraction::default(),
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
