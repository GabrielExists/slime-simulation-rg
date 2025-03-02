use std::f32::consts::PI;
use shared::*;

// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircleFacingInwards {
//     max_distance: 250.0,
// };
// pub const SPAWN_MODE: SpawnMode = SpawnMode::EvenlyDistributed;
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CenterFacingOutwards;
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircumferenceFacingInward { distance: 220.0 };
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircumferenceFacingRandom { distance: 220.0 };
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircumferenceFacingClockwise { distance: 220.0 };
pub const DELTA_TIME: f32 = 1.0 / 24.0;
pub const TIME_SCALE: f32 = 1.0;
pub const COMPUTE_STEPS_PER_RENDER: u32 = 1;
pub const NUM_AGENT_TYPES: usize = 3;
pub const SINGLE_AGENT_STATS: AgentStatsAll = AgentStatsAll {
    spawn_mode: SpawnMode::CircumferenceFacingClockwise { distance: 170.0 },
    num_agents: 2000,
    shader_stats: AgentStats {
        velocity: 30.0,
        turn_speed: PI * 50.0,
        turn_speed_avoidance: PI * 50.0,
        sensor_angle_spacing: PI / 3.0,
        sensor_offset: 4.0,
        pixel_addition: 1.0 / 8.0,
        avoidance_threshold: 10.0,
        evaporation_speed: 40.0,
        diffusion_speed: 180.0,
    },
};
pub const AGENT_STATS: [AgentStatsAll; NUM_AGENT_TYPES] = [
    SINGLE_AGENT_STATS,
    SINGLE_AGENT_STATS,
    SINGLE_AGENT_STATS,
];
