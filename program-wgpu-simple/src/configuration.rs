use shared::AgentStats;

pub const NUM_AGENTS: u32 = 256;
pub const AGENT_STATS: [AgentStats; 1] = [AgentStats {
    // Pixels travelled per second
    velocity: 50.0,
}];
// Percent of full white to black transition per second.
// 100.0 is completely faded after 1 second.
// 50.0 is completely faded after 2 seconds.
pub const EVAPORATION_SPEED: f32 = 20.0;
