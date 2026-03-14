use serde::{Deserialize, Serialize};

/// State of an agent in the simulation.
#[derive(Debug, Clone)]
pub enum AgentState {
    /// Traveling along route: current edge index in route, progress in km along that edge
    Traveling {
        edge_index: usize,
        progress_km: f64,
    },
    /// Arrived at destination
    Arrived,
    /// Waiting to be assigned a route
    Idle,
}

/// Type of route an agent is following.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RouteType {
    Selfish,
    SystemSuggested,
    Alternative,
}

/// Configuration for the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub num_ai_agents: u32,
    pub ai_compliance_rate: f64,
    pub time_scale: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            num_ai_agents: 120,
            ai_compliance_rate: 0.6,
            time_scale: 10.0,
        }
    }
}

/// Snapshot of simulation state returned to JS each tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimSnapshot {
    /// Flat array: [lng0, lat0, lng1, lat1, ...] for all cars
    pub car_positions: Vec<f64>,
    /// 0 = AI, 1 = player
    pub car_types: Vec<u8>,
    /// Current flow on each edge
    pub edge_flows: Vec<f64>,
    /// Simulation time in seconds
    pub game_time: f64,
    /// Player-specific stats
    pub player_progress: f64,
    pub player_speed_kmh: f64,
    pub player_eta_seconds: f64,
    /// Whether player has arrived
    pub player_arrived: bool,
    /// Number of active cars
    pub num_cars: u32,
}

/// A route option presented to the player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteOption {
    pub edge_ids: Vec<u32>,
    pub total_distance_km: f64,
    pub estimated_time_minutes: f64,
    pub token_reward: f64,
    pub route_type: RouteType,
    /// Flat coords for rendering: [[lng, lat], ...]
    pub geometry: Vec<[f64; 2]>,
}
