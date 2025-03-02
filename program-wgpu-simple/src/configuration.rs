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
pub const AGENT_STATS: [AgentStatsAll; NUM_AGENT_TYPES] = [
    AgentStatsAll {
        spawn_mode: SpawnMode::CircumferenceFacingOutward { distance: 170.0 },
        num_agents: 10000,
        shader_stats: AgentStats {
            velocity: 65.0,
            turn_speed: PI * 80.0,
            turn_speed_avoidance: PI * 30.0,
            sensor_angle_spacing: PI / 3.0,
            sensor_offset: 5.0,
            pixel_addition: 1.0 / 5.0,
            avoidance_threshold: 20.0,
            evaporation_speed: 50.0,
            diffusion_speed: 180.0,
            attraction_one: 1.0,
            attraction_two: 1.0,
        },
    },
    AgentStatsAll {
        spawn_mode: SpawnMode::CircumferenceFacingClockwise { distance: 170.0 },
        num_agents: 10000,
        shader_stats: AgentStats {
            velocity: 65.0,
            turn_speed: PI * 80.0,
            turn_speed_avoidance: PI * 30.0,
            sensor_angle_spacing: PI / 3.0,
            sensor_offset: 5.0,
            pixel_addition: 1.0 / 5.0,
            avoidance_threshold: 10.0,
            evaporation_speed: 50.0,
            diffusion_speed: 180.0,
            attraction_one: -1.0,
            attraction_two: 1.0,
        },
    },
];
