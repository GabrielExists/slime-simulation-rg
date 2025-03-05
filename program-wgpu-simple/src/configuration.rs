use shared::*;

pub const DEFAULT_WIDTH: u32 = 800;
pub const DEFAULT_HEIGHT: u32 = 480;
pub const DEFAULT_DISTANCE: u32 = 170;

#[derive(Clone, PartialEq)]
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

#[derive(Clone, PartialEq)]
pub struct Globals {
    pub fixed_delta_time: f32,
    pub time_scale: f32,
    pub compute_steps_per_render: u32,
    pub click_mode: ClickMode,
}

pub const GLOBALS: Globals = Globals {
    fixed_delta_time: 1.0 / 120.0,
    time_scale: 0.2,
    compute_steps_per_render: 1,
    click_mode: ClickMode::PaintTrail(0),
};
pub const AGENT_STATS: [AgentStatsAll; NUM_AGENT_TYPES] = [
    AgentStatsAll {
        name: "Red",
        spawn_mode: SpawnMode::BoxFacingRandom {
            spawn_box: SpawnBox {
                left: 250,
                top: 75,
                box_width: 150,
                box_height: 150,
            }
        },
        num_agents: 4000,
        shader_stats: AgentStats {
            velocity: 65.0,
            pixel_addition: 1.0 / 5.0,
            turn_speed: 80.0,
            turn_speed_avoidance: 30.0,
            avoidance_threshold: 20.0,
            sensor_angle_spacing: 60.0,
            sensor_offset: 5.0,
            attraction_channel_one: 0.2,
            attraction_channel_two: 1.0,
            attraction_channel_three: 0.0,
            attraction_channel_four: -1.0,
        },
    },
    AgentStatsAll {
        name: "Green",
        spawn_mode: SpawnMode::BoxFacingRandom {
            spawn_box: SpawnBox {
                left: 400,
                top: 75,
                box_width: 150,
                box_height: 150,
            }
        },
        num_agents: 4000,
        shader_stats: AgentStats {
            velocity: 65.0,
            pixel_addition: 1.0 / 5.0,
            turn_speed: 80.0,
            turn_speed_avoidance: 30.0,
            avoidance_threshold: 20.0,
            sensor_angle_spacing: 60.0,
            sensor_offset: 5.0,
            attraction_channel_one: -1.0,
            attraction_channel_two: 0.2,
            attraction_channel_three: 1.0,
            attraction_channel_four: 0.0,
        },
    },
    AgentStatsAll {
        name: "Blue",
        spawn_mode: SpawnMode::BoxFacingRandom {
            spawn_box: SpawnBox {
                left: 400,
                top: 225,
                box_width: 150,
                box_height: 150,
            }
        },
        num_agents: 4000,
        shader_stats: AgentStats {
            velocity: 65.0,
            pixel_addition: 1.0 / 5.0,
            turn_speed: 80.0,
            turn_speed_avoidance: 30.0,
            avoidance_threshold: 20.0,
            sensor_angle_spacing: 60.0,
            sensor_offset: 5.0,
            attraction_channel_one: 0.0,
            attraction_channel_two: -1.0,
            attraction_channel_three: 0.2,
            attraction_channel_four: 1.0,
        },
    },
    AgentStatsAll {
        name: "Gray",
        spawn_mode: SpawnMode::BoxFacingRandom {
            spawn_box: SpawnBox {
                left: 250,
                top: 225,
                box_width: 150,
                box_height: 150,
            }
        },
        num_agents: 4000,
        shader_stats: AgentStats {
            velocity: 65.0,
            pixel_addition: 1.0 / 5.0,
            turn_speed: 80.0,
            turn_speed_avoidance: 30.0,
            avoidance_threshold: 20.0,
            sensor_angle_spacing: 60.0,
            sensor_offset: 5.0,
            attraction_channel_one: 1.0,
            attraction_channel_two: 0.0,
            attraction_channel_three: -1.0,
            attraction_channel_four: 0.2,
        },
    },
];

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
