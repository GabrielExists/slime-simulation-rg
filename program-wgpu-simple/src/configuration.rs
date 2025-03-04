use shared::*;

pub const DEFAULT_WIDTH: u32 = 800;
pub const DEFAULT_HEIGHT: u32 = 480;
pub const DEFAULT_DISTANCE: u32 = 170;

pub struct Globals {
    pub fixed_delta_time: f32,
    pub time_scale: f32,
    pub compute_steps_per_render: u32,
    pub click_mode: ClickMode,
}
pub const GLOBALS: Globals = Globals {
    fixed_delta_time: 1.0/120.0,
    time_scale: 1.0,
    compute_steps_per_render: 1,
    click_mode: ClickMode::Disabled,
};
pub const AGENT_STATS: [AgentStatsAll; NUM_AGENT_TYPES] = [
    AgentStatsAll {
        name: "Green",
        spawn_mode: SpawnMode::CircumferenceFacingClockwise { distance: DEFAULT_DISTANCE },
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
            attraction_channel_two: -1.0,
        },
    },
    AgentStatsAll {
        name: "Blue",
        spawn_mode: SpawnMode::CircumferenceFacingClockwise { distance: DEFAULT_DISTANCE },
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
            attraction_channel_two: 0.2,
        },
    },
];

pub const TRAIL_NAMES: [&'static str; NUM_TRAIL_STATS] = [
    "Green",
    "Blue",
];

pub const TRAIL_STATS: [TrailStats; NUM_TRAIL_STATS] = [
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 180.0,
    },
    TrailStats {
        evaporation_speed: 50.0,
        diffusion_speed: 180.0,
    },
];
