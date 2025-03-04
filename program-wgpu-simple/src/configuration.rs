use shared::*;

// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircleFacingInwards {
//     max_distance: 250.0,
// };
// pub const SPAWN_MODE: SpawnMode = SpawnMode::EvenlyDistributed;
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CenterFacingOutwards;
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircumferenceFacingInward { distance: 220.0 };
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircumferenceFacingOutward { distance: 170.0 };
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircumferenceFacingRandom { distance: 220.0 };
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircumferenceFacingClockwise { distance: 220.0 };
pub struct Globals {
    pub delta_time: f32,
    pub time_scale: f32,
    pub compute_steps_per_render: u32,
}
pub const GLOBALS: Globals = Globals {
    delta_time: 1.0/60.0,
    time_scale: 1.0,
    compute_steps_per_render: 1,
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
    // TrailStats {
    //     evaporation_speed: 50.0,
    //     diffusion_speed: 180.0,
    // },
];
