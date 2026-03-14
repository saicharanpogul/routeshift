use routeshift_core::bpr;
use routeshift_core::dijkstra;
use routeshift_core::frank_wolfe;
use routeshift_core::graph::Graph;
use routeshift_core::types::{
    AssignmentType, FrankWolfeConfig, ODPair, RoadNetwork,
};

use crate::agent::Agent;
use crate::types::{AgentState, RouteOption, RouteType, SimConfig, SimSnapshot};

/// Simple xorshift64 RNG to avoid depending on `rand` crate.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(if seed == 0 { 0xDEADBEEF } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Random f64 in [0, 1)
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Random usize in [0, max)
    fn next_usize(&mut self, max: usize) -> usize {
        (self.next_u64() as usize) % max
    }
}

/// The core simulation engine.
pub struct Simulation {
    graph: Graph,
    od_pairs: Vec<ODPair>,
    agents: Vec<Agent>,
    /// Current flow on each edge (count of agents)
    edge_flows: Vec<f64>,
    /// Coordinates for each edge: edge_geometries[edge_id] = [[lng, lat], ...]
    edge_geometries: Vec<Vec<[f64; 2]>>,
    /// Length of each edge in km
    edge_lengths: Vec<f64>,
    /// Precomputed system-optimal flows for reward calculation
    so_flows: Vec<f64>,
    /// Demand scaling factor: total_demand / num_ai_agents
    demand_scale: f64,
    config: SimConfig,
    game_time: f64,
    next_agent_id: u32,
    player_agent_id: Option<u32>,
    /// Cached route options for the player
    cached_route_options: Vec<RouteOption>,
    rng: Rng,
}

impl Simulation {
    pub fn new(
        network: &RoadNetwork,
        config: SimConfig,
        edge_geometries: Vec<Vec<[f64; 2]>>,
    ) -> Self {
        let graph = Graph::from_network(network);
        let num_edges = graph.num_edges;

        let edge_lengths: Vec<f64> = network.edges.iter().map(|e| e.length_km).collect();

        // Compute system-optimal flows for reward calculation
        let so_config = FrankWolfeConfig {
            max_iterations: 50,
            convergence_threshold: 0.01,
            assignment_type: AssignmentType::SystemOptimal,
        };
        let so_result = frank_wolfe::solve(&graph, &network.od_pairs, &so_config);
        let so_flows = so_result.edge_flows;

        // Demand scale: how much demand each AI agent represents
        let total_demand: f64 = network.od_pairs.iter().map(|od| od.demand).sum();
        let demand_scale = if config.num_ai_agents > 0 {
            total_demand / config.num_ai_agents as f64
        } else {
            1.0
        };

        let mut sim = Simulation {
            graph,
            od_pairs: network.od_pairs.clone(),
            agents: Vec::with_capacity(config.num_ai_agents as usize + 1),
            edge_flows: vec![0.0; num_edges],
            edge_geometries,
            edge_lengths,
            so_flows,
            demand_scale,
            config,
            game_time: 0.0,
            next_agent_id: 0,
            player_agent_id: None,
            cached_route_options: Vec::new(),
            rng: Rng::new(42),
        };

        // Spawn initial AI agents
        let num_to_spawn = sim.config.num_ai_agents;
        for _ in 0..num_to_spawn {
            sim.spawn_ai_agent();
        }

        sim
    }

    /// Spawn a new AI agent with a random OD pair and route.
    fn spawn_ai_agent(&mut self) {
        if self.od_pairs.is_empty() {
            return;
        }

        let od_idx = self.rng.next_usize(self.od_pairs.len());
        let od = &self.od_pairs[od_idx];

        // Decide if this agent is compliant (follows SO route)
        let compliant = self.rng.next_f64() < self.config.ai_compliance_rate;

        // Compute route: use current edge costs
        let edge_costs = self.compute_current_costs(if compliant {
            AssignmentType::SystemOptimal
        } else {
            AssignmentType::UserEquilibrium
        });

        let (_, predecessors) =
            dijkstra::shortest_path_tree(&self.graph, od.origin, &edge_costs);

        if let Some(route) =
            dijkstra::reconstruct_path(&self.graph, &predecessors, od.destination)
        {
            let route_edge_lengths: Vec<f64> =
                route.iter().map(|&eid| self.edge_lengths[eid as usize]).collect();

            let agent = Agent::new(
                self.next_agent_id,
                od.origin,
                od.destination,
                route,
                route_edge_lengths,
                compliant,
                false,
            );
            self.next_agent_id += 1;
            self.agents.push(agent);
        }
    }

    /// Compute edge costs based on current flows.
    fn compute_current_costs(&self, assignment_type: AssignmentType) -> Vec<f64> {
        self.graph
            .edges
            .iter()
            .zip(self.edge_flows.iter())
            .map(|(edge, &flow)| {
                let scaled_flow = flow * self.demand_scale;
                match assignment_type {
                    AssignmentType::UserEquilibrium => {
                        bpr::travel_time(edge.free_flow_time, scaled_flow, edge.capacity)
                    }
                    AssignmentType::SystemOptimal => {
                        bpr::marginal_cost(edge.free_flow_time, scaled_flow, edge.capacity)
                    }
                }
            })
            .collect()
    }

    /// Compute speed in km/h for each edge based on current flows.
    fn compute_edge_speeds(&self) -> Vec<f64> {
        self.graph
            .edges
            .iter()
            .zip(self.edge_flows.iter())
            .map(|(edge, &flow)| {
                let scaled_flow = flow * self.demand_scale;
                let travel_time_min =
                    bpr::travel_time(edge.free_flow_time, scaled_flow, edge.capacity);
                if travel_time_min > 0.0 && edge.length_km > 0.0 {
                    (edge.length_km / travel_time_min) * 60.0 // km/h
                } else {
                    30.0 // fallback
                }
            })
            .collect()
    }

    /// Run one simulation tick.
    pub fn tick(&mut self, dt_seconds: f64) -> SimSnapshot {
        let dt = dt_seconds * self.config.time_scale;
        self.game_time += dt;

        // Compute edge speeds from current flows
        let edge_speeds = self.compute_edge_speeds();

        // Recount edge flows
        self.edge_flows.fill(0.0);
        for agent in &self.agents {
            if let Some(eid) = agent.current_edge() {
                self.edge_flows[eid as usize] += 1.0;
            }
        }

        // Advance all agents
        let mut arrived_indices = Vec::new();
        for (i, agent) in self.agents.iter_mut().enumerate() {
            if agent.tick(dt, &edge_speeds) {
                arrived_indices.push(i);
            }
        }

        // Collect player arrival status
        let player_arrived = self
            .player_agent_id
            .map(|pid| arrived_indices.iter().any(|&i| self.agents[i].id == pid))
            .unwrap_or(false);

        // Respawn arrived AI agents (not the player)
        for &idx in arrived_indices.iter().rev() {
            let agent = &self.agents[idx];
            if !agent.is_player {
                self.agents.swap_remove(idx);
            }
        }

        // Spawn replacements to maintain target count
        let ai_count = self.agents.iter().filter(|a| !a.is_player).count();
        let target = self.config.num_ai_agents as usize;
        for _ in ai_count..target {
            self.spawn_ai_agent();
        }

        // Build snapshot
        let mut car_positions = Vec::with_capacity(self.agents.len() * 2);
        let mut car_types = Vec::with_capacity(self.agents.len());

        let mut player_progress = 0.0;
        let mut player_speed = 0.0;
        let mut player_eta = 0.0;

        for agent in &self.agents {
            match &agent.state {
                AgentState::Arrived if !agent.is_player => continue,
                AgentState::Idle => continue,
                _ => {}
            }

            let pos = agent.position(&self.edge_geometries);
            car_positions.push(pos[0]);
            car_positions.push(pos[1]);
            car_types.push(if agent.is_player { 1 } else { 0 });

            if agent.is_player {
                player_progress = agent.route_progress();
                player_speed = agent.current_speed(&edge_speeds);
                player_eta = agent.eta_seconds(&edge_speeds);
            }
        }

        // Compute display edge flows (scaled)
        let display_flows: Vec<f64> = self
            .edge_flows
            .iter()
            .map(|&f| f * self.demand_scale)
            .collect();

        let num_cars = car_types.len() as u32;
        SimSnapshot {
            car_positions,
            car_types,
            edge_flows: display_flows,
            game_time: self.game_time,
            player_progress,
            player_speed_kmh: player_speed,
            player_eta_seconds: player_eta,
            player_arrived,
            num_cars,
        }
    }

    /// Spawn the player agent at a given origin node (without a route yet).
    pub fn spawn_player(&mut self, origin: u32) {
        // Remove existing player agent if any
        if let Some(pid) = self.player_agent_id {
            self.agents.retain(|a| a.id != pid);
        }

        let agent = Agent::new(
            self.next_agent_id,
            origin,
            origin, // destination set later
            Vec::new(),
            Vec::new(),
            true,
            true,
        );
        self.player_agent_id = Some(self.next_agent_id);
        self.next_agent_id += 1;
        self.agents.push(agent);
    }

    /// Compute route options from player's current origin to a destination.
    pub fn compute_route_options(
        &mut self,
        origin: u32,
        destination: u32,
    ) -> Vec<RouteOption> {
        let mut options = Vec::new();

        // 1. Selfish route (current travel times)
        let ue_costs = self.compute_current_costs(AssignmentType::UserEquilibrium);
        let (_, ue_pred) = dijkstra::shortest_path_tree(&self.graph, origin, &ue_costs);
        if let Some(selfish_route) =
            dijkstra::reconstruct_path(&self.graph, &ue_pred, destination)
        {
            let option =
                self.build_route_option(&selfish_route, &ue_costs, RouteType::Selfish);
            options.push(option);

            // 2. System-suggested route (marginal costs)
            let so_costs = self.compute_current_costs(AssignmentType::SystemOptimal);
            let (_, so_pred) =
                dijkstra::shortest_path_tree(&self.graph, origin, &so_costs);
            if let Some(so_route) =
                dijkstra::reconstruct_path(&self.graph, &so_pred, destination)
            {
                if so_route != selfish_route {
                    let option = self.build_route_option(
                        &so_route,
                        &ue_costs,
                        RouteType::SystemSuggested,
                    );
                    options.push(option);
                }
            }

            // 3. Alternative route (penalize selfish edges)
            let mut alt_costs = ue_costs.clone();
            for &eid in &selfish_route {
                alt_costs[eid as usize] *= 3.0;
            }
            let (_, alt_pred) =
                dijkstra::shortest_path_tree(&self.graph, origin, &alt_costs);
            if let Some(alt_route) =
                dijkstra::reconstruct_path(&self.graph, &alt_pred, destination)
            {
                if alt_route != selfish_route
                    && options
                        .iter()
                        .all(|o| o.edge_ids != alt_route)
                {
                    let option = self.build_route_option(
                        &alt_route,
                        &ue_costs,
                        RouteType::Alternative,
                    );
                    options.push(option);
                }
            }
        }

        // Compute token rewards
        self.assign_token_rewards(&mut options);

        self.cached_route_options = options.clone();
        options
    }

    /// Build a RouteOption from a sequence of edge IDs.
    fn build_route_option(
        &self,
        route: &[u32],
        travel_costs: &[f64],
        route_type: RouteType,
    ) -> RouteOption {
        let total_distance_km: f64 = route
            .iter()
            .map(|&eid| self.edge_lengths[eid as usize])
            .sum();

        let estimated_time_minutes: f64 = route
            .iter()
            .map(|&eid| travel_costs[eid as usize])
            .sum();

        // Build geometry from edge geometries
        let mut geometry = Vec::new();
        for &eid in route {
            let coords = &self.edge_geometries[eid as usize];
            if geometry.is_empty() {
                geometry.extend_from_slice(coords);
            } else if !coords.is_empty() {
                // Skip first point if it matches last point (avoid duplication)
                let start = if !geometry.is_empty()
                    && (geometry.last().unwrap()[0] - coords[0][0]).abs() < 1e-8
                    && (geometry.last().unwrap()[1] - coords[0][1]).abs() < 1e-8
                {
                    1
                } else {
                    0
                };
                geometry.extend_from_slice(&coords[start..]);
            }
        }

        RouteOption {
            edge_ids: route.to_vec(),
            total_distance_km,
            estimated_time_minutes,
            token_reward: 0.0, // set later
            route_type,
            geometry,
        }
    }

    /// Assign token rewards based on system benefit.
    fn assign_token_rewards(&self, options: &mut [RouteOption]) {
        if options.is_empty() {
            return;
        }

        let base_reward = 10.0;
        let selfish_time = options[0].estimated_time_minutes;

        for i in 0..options.len() {
            let reward_multiplier = match options[i].route_type {
                RouteType::Selfish => 1.0,
                RouteType::SystemSuggested => {
                    let time_ratio = if selfish_time > 0.0 {
                        (options[i].estimated_time_minutes / selfish_time).max(1.0)
                    } else {
                        1.0
                    };
                    1.0 + (time_ratio - 1.0) * 5.0 + 1.5
                }
                RouteType::Alternative => 1.5,
            };
            options[i].token_reward = (base_reward * reward_multiplier).round();
        }
    }

    /// Set the player's route from the cached options.
    pub fn set_player_route(&mut self, route_index: usize) {
        if route_index >= self.cached_route_options.len() {
            return;
        }

        let option = &self.cached_route_options[route_index];
        let route = option.edge_ids.clone();
        let route_edge_lengths: Vec<f64> = route
            .iter()
            .map(|&eid| self.edge_lengths[eid as usize])
            .collect();

        if let Some(pid) = self.player_agent_id {
            if let Some(agent) = self.agents.iter_mut().find(|a| a.id == pid) {
                agent.destination = if let Some(&last_eid) = route.last() {
                    self.graph.edges[last_eid as usize].target
                } else {
                    agent.origin
                };
                agent.route = route;
                agent.route_edge_lengths = route_edge_lengths;
                agent.compliant = option.route_type == RouteType::SystemSuggested;
                agent.state = AgentState::Traveling {
                    edge_index: 0,
                    progress_km: 0.0,
                };
            }
        }
    }

    /// Get the token reward for a cached route option.
    pub fn get_route_reward(&self, route_index: usize) -> f64 {
        self.cached_route_options
            .get(route_index)
            .map(|o| o.token_reward)
            .unwrap_or(0.0)
    }

    /// Find the nearest node to a given [lng, lat] coordinate.
    pub fn find_nearest_node(&self, lng: f64, lat: f64) -> u32 {
        let mut best_id = 0u32;
        let mut best_dist = f64::MAX;

        for node in &self.graph.edges {
            // We don't have direct node coords in Graph, use edges to approximate
            let _ = node;
        }

        // Fallback: scan OD pairs for valid nodes and check against edge endpoints
        // Actually we need the node positions. Let's iterate via edge source/target
        // and use edge_geometries to get coords.
        let mut seen = std::collections::HashSet::new();
        for (eid, geom) in self.edge_geometries.iter().enumerate() {
            if eid >= self.graph.edges.len() {
                break;
            }
            let edge = &self.graph.edges[eid];

            // Source node
            if !seen.contains(&edge.source) && !geom.is_empty() {
                let coord = geom[0];
                let dist = (coord[0] - lng).powi(2) + (coord[1] - lat).powi(2);
                if dist < best_dist {
                    best_dist = dist;
                    best_id = edge.source;
                }
                seen.insert(edge.source);
            }

            // Target node
            if !seen.contains(&edge.target) && !geom.is_empty() {
                let coord = *geom.last().unwrap();
                let dist = (coord[0] - lng).powi(2) + (coord[1] - lat).powi(2);
                if dist < best_dist {
                    best_dist = dist;
                    best_id = edge.target;
                }
                seen.insert(edge.target);
            }
        }

        best_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use routeshift_core::types::{Edge, Node};

    fn make_test_network() -> (RoadNetwork, Vec<Vec<[f64; 2]>>) {
        let network = RoadNetwork {
            nodes: vec![
                Node { id: 0, lat: 17.0, lon: 78.0 },
                Node { id: 1, lat: 17.01, lon: 78.0 },
                Node { id: 2, lat: 17.0, lon: 78.01 },
                Node { id: 3, lat: 17.01, lon: 78.01 },
            ],
            edges: vec![
                Edge { id: 0, source: 0, target: 1, free_flow_time: 2.0, capacity: 100.0, length_km: 1.0 },
                Edge { id: 1, source: 0, target: 2, free_flow_time: 3.0, capacity: 200.0, length_km: 1.5 },
                Edge { id: 2, source: 1, target: 3, free_flow_time: 3.0, capacity: 200.0, length_km: 1.5 },
                Edge { id: 3, source: 2, target: 3, free_flow_time: 2.0, capacity: 100.0, length_km: 1.0 },
            ],
            od_pairs: vec![
                ODPair { origin: 0, destination: 3, demand: 5.0 },
            ],
        };

        let geometries = vec![
            vec![[78.0, 17.0], [78.0, 17.01]],
            vec![[78.0, 17.0], [78.01, 17.0]],
            vec![[78.0, 17.01], [78.01, 17.01]],
            vec![[78.01, 17.0], [78.01, 17.01]],
        ];

        (network, geometries)
    }

    #[test]
    fn test_simulation_creates() {
        let (network, geometries) = make_test_network();
        let config = SimConfig {
            num_ai_agents: 10,
            ai_compliance_rate: 0.5,
            time_scale: 1.0,
        };
        let sim = Simulation::new(&network, config, geometries);
        assert_eq!(sim.agents.len(), 10);
    }

    #[test]
    fn test_simulation_tick() {
        let (network, geometries) = make_test_network();
        let config = SimConfig {
            num_ai_agents: 5,
            ai_compliance_rate: 0.5,
            time_scale: 1.0,
        };
        let mut sim = Simulation::new(&network, config, geometries);
        let snapshot = sim.tick(1.0);
        assert!(snapshot.num_cars > 0);
        assert!(!snapshot.car_positions.is_empty());
    }

    #[test]
    fn test_route_options() {
        let (network, geometries) = make_test_network();
        let config = SimConfig {
            num_ai_agents: 5,
            ai_compliance_rate: 0.5,
            time_scale: 1.0,
        };
        let mut sim = Simulation::new(&network, config, geometries);
        let options = sim.compute_route_options(0, 3);
        assert!(!options.is_empty());
        assert!(options[0].total_distance_km > 0.0);
    }
}
