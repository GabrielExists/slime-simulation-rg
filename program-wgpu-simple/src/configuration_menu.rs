use crate::configuration::Globals;
use shared::{AgentStatsAll, NUM_AGENT_TYPES, NUM_TRAIL_STATS, TrailStats};

pub struct ConfigurationValues {
    pub globals: Globals,
    pub agent_stats: [AgentStatsAll; NUM_AGENT_TYPES],
    pub trail_stats: [TrailStats; NUM_TRAIL_STATS],
}
