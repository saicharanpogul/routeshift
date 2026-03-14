use serde::{Deserialize, Serialize};

/// A node in the road network graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u32,
    pub lat: f64,
    pub lon: f64,
}

/// A directed edge in the road network graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: u32,
    pub source: u32,
    pub target: u32,
    pub free_flow_time: f64,
    pub capacity: f64,
    pub length_km: f64,
}

/// An origin-destination pair with demand (number of vehicles).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ODPair {
    pub origin: u32,
    pub destination: u32,
    pub demand: f64,
}

/// Complete road network data passed from JS to WASM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadNetwork {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub od_pairs: Vec<ODPair>,
}

/// Whether to solve for User Equilibrium or System Optimal.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AssignmentType {
    UserEquilibrium,
    SystemOptimal,
}

/// Configuration for the Frank-Wolfe solver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrankWolfeConfig {
    pub max_iterations: u32,
    pub convergence_threshold: f64,
    pub assignment_type: AssignmentType,
}

impl Default for FrankWolfeConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 0.01,
            assignment_type: AssignmentType::UserEquilibrium,
        }
    }
}

/// Result of a traffic assignment computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentResult {
    pub edge_flows: Vec<f64>,
    pub edge_travel_times: Vec<f64>,
    pub total_system_travel_time: f64,
    pub iterations: u32,
    pub converged: bool,
    pub relative_gap: f64,
}
