use std::cell::RefCell;
use wasm_bindgen::prelude::*;

use routeshift_core::frank_wolfe;
use routeshift_core::graph::Graph;
use routeshift_core::types::{FrankWolfeConfig, RoadNetwork};
use routeshift_sim::simulation::Simulation;
use routeshift_sim::types::SimConfig;

thread_local! {
    static SIM: RefCell<Option<Simulation>> = RefCell::new(None);
}

// ─── Original API (kept for backward compat) ───

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
    let (_, predecessors) =
        routeshift_core::dijkstra::shortest_path_tree(&graph, source, &costs);

    match routeshift_core::dijkstra::reconstruct_path(&graph, &predecessors, target) {
        Some(path) => serde_wasm_bindgen::to_value(&path)
            .map_err(|e| JsError::new(&format!("Failed to serialize path: {}", e))),
        None => Ok(JsValue::NULL),
    }
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ─── Simulation API ───

/// Initialize the simulation with a road network, config, and edge geometries.
///
/// `edge_geometries_js` is an array of arrays of [lng, lat] pairs, indexed by edge_id.
#[wasm_bindgen]
pub fn init_simulation(
    network_js: JsValue,
    config_js: JsValue,
    edge_geometries_js: JsValue,
) -> Result<(), JsError> {
    let network: RoadNetwork = serde_wasm_bindgen::from_value(network_js)
        .map_err(|e| JsError::new(&format!("Failed to parse network: {}", e)))?;
    let config: SimConfig = serde_wasm_bindgen::from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse sim config: {}", e)))?;
    let edge_geometries: Vec<Vec<[f64; 2]>> =
        serde_wasm_bindgen::from_value(edge_geometries_js)
            .map_err(|e| JsError::new(&format!("Failed to parse geometries: {}", e)))?;

    let sim = Simulation::new(&network, config, edge_geometries);

    SIM.with(|s| {
        *s.borrow_mut() = Some(sim);
    });

    Ok(())
}

/// Run one simulation tick and return a snapshot.
#[wasm_bindgen]
pub fn sim_tick(dt_seconds: f64) -> Result<JsValue, JsError> {
    SIM.with(|s| {
        let mut sim = s.borrow_mut();
        let sim = sim
            .as_mut()
            .ok_or_else(|| JsError::new("Simulation not initialized"))?;

        let snapshot = sim.tick(dt_seconds);

        serde_wasm_bindgen::to_value(&snapshot)
            .map_err(|e| JsError::new(&format!("Failed to serialize snapshot: {}", e)))
    })
}

/// Spawn the player agent at a given origin node.
#[wasm_bindgen]
pub fn spawn_player(origin: u32) -> Result<(), JsError> {
    SIM.with(|s| {
        let mut sim = s.borrow_mut();
        let sim = sim
            .as_mut()
            .ok_or_else(|| JsError::new("Simulation not initialized"))?;
        sim.spawn_player(origin);
        Ok(())
    })
}

/// Compute route options from origin to destination.
/// Returns an array of RouteOption objects.
#[wasm_bindgen]
pub fn compute_route_options(origin: u32, destination: u32) -> Result<JsValue, JsError> {
    SIM.with(|s| {
        let mut sim = s.borrow_mut();
        let sim = sim
            .as_mut()
            .ok_or_else(|| JsError::new("Simulation not initialized"))?;

        let options = sim.compute_route_options(origin, destination);

        serde_wasm_bindgen::to_value(&options)
            .map_err(|e| JsError::new(&format!("Failed to serialize options: {}", e)))
    })
}

/// Set the player's route from the cached route options.
#[wasm_bindgen]
pub fn set_player_route(route_index: u32) -> Result<(), JsError> {
    SIM.with(|s| {
        let mut sim = s.borrow_mut();
        let sim = sim
            .as_mut()
            .ok_or_else(|| JsError::new("Simulation not initialized"))?;
        sim.set_player_route(route_index as usize);
        Ok(())
    })
}

/// Get token reward for a specific route option.
#[wasm_bindgen]
pub fn get_route_reward(route_index: u32) -> Result<f64, JsError> {
    SIM.with(|s| {
        let sim = s.borrow();
        let sim = sim
            .as_ref()
            .ok_or_else(|| JsError::new("Simulation not initialized"))?;
        Ok(sim.get_route_reward(route_index as usize))
    })
}

/// Find the nearest network node to a given [lng, lat] coordinate.
#[wasm_bindgen]
pub fn find_nearest_node(lng: f64, lat: f64) -> Result<u32, JsError> {
    SIM.with(|s| {
        let sim = s.borrow();
        let sim = sim
            .as_ref()
            .ok_or_else(|| JsError::new("Simulation not initialized"))?;
        Ok(sim.find_nearest_node(lng, lat))
    })
}
