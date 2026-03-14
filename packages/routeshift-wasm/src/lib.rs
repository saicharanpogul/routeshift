use wasm_bindgen::prelude::*;

use routeshift_core::frank_wolfe;
use routeshift_core::graph::Graph;
use routeshift_core::types::{FrankWolfeConfig, RoadNetwork};

/// Solve the traffic assignment problem.
///
/// Takes a road network and solver configuration as JS objects,
/// returns an AssignmentResult as a JS object.
#[wasm_bindgen]
pub fn solve_traffic_assignment(
    network_js: JsValue,
    config_js: JsValue,
) -> Result<JsValue, JsError> {
    let network: RoadNetwork = serde_wasm_bindgen::from_value(network_js)
        .map_err(|e| JsError::new(&format!("Failed to parse network: {}", e)))?;
    let config: FrankWolfeConfig = serde_wasm_bindgen::from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse config: {}", e)))?;

    let graph = Graph::from_network(&network);
    let result = frank_wolfe::solve(&graph, &network.od_pairs, &config);

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Compute shortest path between two nodes.
///
/// Returns the path as an array of edge IDs, or null if unreachable.
#[wasm_bindgen]
pub fn compute_shortest_path(
    network_js: JsValue,
    source: u32,
    target: u32,
) -> Result<JsValue, JsError> {
    let network: RoadNetwork = serde_wasm_bindgen::from_value(network_js)
        .map_err(|e| JsError::new(&format!("Failed to parse network: {}", e)))?;

    let graph = Graph::from_network(&network);
    let costs: Vec<f64> = graph.edges.iter().map(|e| e.free_flow_time).collect();
    let (_, predecessors) = routeshift_core::dijkstra::shortest_path_tree(&graph, source, &costs);

    match routeshift_core::dijkstra::reconstruct_path(&graph, &predecessors, target) {
        Some(path) => serde_wasm_bindgen::to_value(&path)
            .map_err(|e| JsError::new(&format!("Failed to serialize path: {}", e))),
        None => Ok(JsValue::NULL),
    }
}

/// Get the version of the WASM module.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
