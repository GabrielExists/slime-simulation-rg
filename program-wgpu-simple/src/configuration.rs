use std::f32::consts::PI;
use shared::AgentStats;

#[allow(dead_code)]
pub enum SpawnMode {
    EvenlyDistributed,
    CenterFacingOutwards,
    PointFacingOutwards {
        x: f32,
        y: f32,
    },
    CircleFacingInwards {
        max_distance: f32,
    },

}
// pub const SPAWN_MODE: SpawnMode = SpawnMode::CircleFacingInwards {
//     max_distance: 200.0,
// };
pub const SPAWN_MODE: SpawnMode = SpawnMode::EvenlyDistributed;
pub const TIME_SCALE: f32 = 1.0;
pub const COMPUTE_STEPS_PER_RENDER: u32 = 1;
pub const NUM_AGENTS: u32 = 10000;
pub const AGENT_STATS: [AgentStats; 1] = [AgentStats {
    // Pixels travelled per second
    velocity: 50.0,
    turn_speed: PI * 10.0,
    sensor_angle_spacing: PI / 3.0,
    sensor_offset: 5.0,
    pixel_addition: 0x0FFFFFFF,
}];
// Percent of full white to black transition per second.
// 100.0 is completely faded after 1 second.
// 50.0 is completely faded after 2 seconds.
pub const EVAPORATION_SPEED: f32 = 80.0;

// Speed of diffusion in percent.
// Reaching 90% takes 1 second if set to 240%.
// Reaching 86% takes 1 second if set to 200%.
// Reaching 63% takes 1 second if set to 100%.
pub const DIFFUSION_SPEED: f32 = 180.0;
