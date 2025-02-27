use shared::AgentStats;

pub const NUM_AGENTS: u32 = 10000;
pub const AGENT_STATS: [AgentStats; 1] = [AgentStats {
    // Pixels travelled per second
    velocity: 50.0,
}];
// Percent of full white to black transition per second.
// 100.0 is completely faded after 1 second.
// 50.0 is completely faded after 2 seconds.
pub const EVAPORATION_SPEED: f32 = 30.0;

// Speed of diffusion in percent.
// Reaching 90% takes 1 second if set to 240%.
// Reaching 86% takes 1 second if set to 200%.
// Reaching 63% takes 1 second if set to 100%.
pub const DIFFUSION_SPEED: f32 = 600.0;
