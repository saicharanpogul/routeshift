use crate::bpr;
use crate::dijkstra;
use crate::graph::Graph;
use crate::types::{AssignmentResult, AssignmentType, Edge, FrankWolfeConfig, ODPair};

/// Compute edge costs (travel times or marginal costs) based on current flows.
fn compute_edge_costs(
    edges: &[Edge],
    flows: &[f64],
    assignment_type: AssignmentType,
) -> Vec<f64> {
    edges
        .iter()
        .zip(flows.iter())
        .map(|(edge, &flow)| match assignment_type {
            AssignmentType::UserEquilibrium => {
                bpr::travel_time(edge.free_flow_time, flow, edge.capacity)
            }
            AssignmentType::SystemOptimal => {
                bpr::marginal_cost(edge.free_flow_time, flow, edge.capacity)
            }
        })
        .collect()
}

/// Perform all-or-nothing assignment: for each OD pair, find shortest path
/// and assign all demand to that path.
fn all_or_nothing(
    graph: &Graph,
    od_pairs: &[ODPair],
    edge_costs: &[f64],
) -> Vec<f64> {
    let mut auxiliary_flows = vec![0.0; graph.num_edges];

    // Group OD pairs by origin for efficiency (one Dijkstra per unique origin)
    let mut origins: std::collections::HashMap<u32, Vec<&ODPair>> = std::collections::HashMap::new();
    for od in od_pairs {
        origins.entry(od.origin).or_default().push(od);
    }

    for (origin, pairs) in &origins {
        let (_, predecessors) = dijkstra::shortest_path_tree(graph, *origin, edge_costs);

        for od in pairs {
            if let Some(path_edges) = dijkstra::reconstruct_path(graph, &predecessors, od.destination) {
                for edge_id in path_edges {
                    auxiliary_flows[edge_id as usize] += od.demand;
                }
            }
        }
    }

    auxiliary_flows
}

/// Line search: find optimal step size alpha in [0, 1] using bisection.
///
/// We minimize: sum_e integral_0^{x_e + alpha*(y_e - x_e)} l_e(w) dw
/// which is the Beckmann objective evaluated at x + alpha*(y - x).
fn line_search(
    edges: &[Edge],
    current_flows: &[f64],
    auxiliary_flows: &[f64],
) -> f64 {
    let mut lo = 0.0_f64;
    let mut hi = 1.0_f64;

    for _ in 0..20 {
        let mid = (lo + hi) / 2.0;
        let mid_plus = mid + 1e-8;

        let obj_mid = evaluate_beckmann(edges, current_flows, auxiliary_flows, mid);
        let obj_mid_plus = evaluate_beckmann(edges, current_flows, auxiliary_flows, mid_plus);

        if obj_mid_plus < obj_mid {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    (lo + hi) / 2.0
}

/// Evaluate the Beckmann objective at flows = current + alpha * (auxiliary - current).
fn evaluate_beckmann(
    edges: &[Edge],
    current_flows: &[f64],
    auxiliary_flows: &[f64],
    alpha: f64,
) -> f64 {
    let mut total = 0.0;
    for (i, edge) in edges.iter().enumerate() {
        let flow = current_flows[i] + alpha * (auxiliary_flows[i] - current_flows[i]);
        total += bpr::beckmann_integral(edge.free_flow_time, flow, edge.capacity);
    }
    total
}

/// Compute the relative gap for convergence checking.
///
/// Gap = |sum_e l_e(x) * (y_e - x_e)| / |sum_e l_e(x) * x_e|
fn compute_relative_gap(
    edge_costs: &[f64],
    current_flows: &[f64],
    auxiliary_flows: &[f64],
) -> f64 {
    let numerator: f64 = edge_costs
        .iter()
        .zip(current_flows.iter())
        .zip(auxiliary_flows.iter())
        .map(|((&cost, &x), &y)| cost * (y - x))
        .sum::<f64>()
        .abs();

    let denominator: f64 = edge_costs
        .iter()
        .zip(current_flows.iter())
        .map(|(&cost, &x)| cost * x)
        .sum::<f64>()
        .abs();

    if denominator < 1e-10 {
        return 1.0;
    }

    numerator / denominator
}

/// Solve the traffic assignment problem using the Frank-Wolfe algorithm.
///
/// Returns an AssignmentResult with edge flows, travel times, and convergence info.
pub fn solve(
    graph: &Graph,
    od_pairs: &[ODPair],
    config: &FrankWolfeConfig,
) -> AssignmentResult {
    let edges = &graph.edges;
    let num_edges = graph.num_edges;

    // Step 1: Initialize with all-or-nothing assignment using free-flow times
    let free_flow_costs: Vec<f64> = edges.iter().map(|e| e.free_flow_time).collect();
    let mut flows = all_or_nothing(graph, od_pairs, &free_flow_costs);

    let mut iterations = 0;
    let mut converged = false;
    let mut relative_gap = 1.0;

    // Step 2: Iterative improvement
    for iter in 0..config.max_iterations {
        iterations = iter + 1;

        // 2a: Compute edge costs with current flows
        let edge_costs = compute_edge_costs(edges, &flows, config.assignment_type);

        // 2b: All-or-nothing assignment with current costs
        let auxiliary_flows = all_or_nothing(graph, od_pairs, &edge_costs);

        // 2c: Check convergence
        relative_gap = compute_relative_gap(&edge_costs, &flows, &auxiliary_flows);
        if relative_gap < config.convergence_threshold {
            converged = true;
            break;
        }

        // 2d: Line search for optimal step size
        let alpha = line_search(edges, &flows, &auxiliary_flows);

        // 2e: Update flows
        for e in 0..num_edges {
            flows[e] += alpha * (auxiliary_flows[e] - flows[e]);
        }
    }

    // Compute final travel times (actual BPR, not marginal)
    let edge_travel_times: Vec<f64> = edges
        .iter()
        .zip(flows.iter())
        .map(|(edge, &flow)| bpr::travel_time(edge.free_flow_time, flow, edge.capacity))
        .collect();

    let total_system_travel_time: f64 = edge_travel_times
        .iter()
        .zip(flows.iter())
        .map(|(&tt, &flow)| tt * flow)
        .sum();

    AssignmentResult {
        edge_flows: flows,
        edge_travel_times,
        total_system_travel_time,
        iterations,
        converged,
        relative_gap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Node, RoadNetwork};

    /// Braess network:
    ///   0 -> 1 (cost: 10x)
    ///   0 -> 2 (cost: 50)
    ///   1 -> 2 (cost: 10)  -- the "Braess edge"
    ///   1 -> 3 (cost: 50)
    ///   2 -> 3 (cost: 10x)
    ///
    /// With 6 units of demand from 0 to 3.
    /// Without Braess edge: UE sends 3 via 0->1->3 and 3 via 0->2->3, total cost = 498
    fn make_braess_network() -> RoadNetwork {
        RoadNetwork {
            nodes: vec![
                Node { id: 0, lat: 0.0, lon: 0.0 },
                Node { id: 1, lat: 1.0, lon: 0.0 },
                Node { id: 2, lat: 0.0, lon: 1.0 },
                Node { id: 3, lat: 1.0, lon: 1.0 },
            ],
            edges: vec![
                // 0->1: linear latency, free_flow = 1.0, capacity = 10.0 (so at flow x, time ~ x for BPR)
                Edge { id: 0, source: 0, target: 1, free_flow_time: 1.0, capacity: 10.0, length_km: 1.0 },
                // 0->2: constant-ish latency (high capacity)
                Edge { id: 1, source: 0, target: 2, free_flow_time: 5.0, capacity: 1000.0, length_km: 2.0 },
                // 1->3: constant-ish latency (high capacity)
                Edge { id: 2, source: 1, target: 3, free_flow_time: 5.0, capacity: 1000.0, length_km: 2.0 },
                // 2->3: linear latency
                Edge { id: 3, source: 2, target: 3, free_flow_time: 1.0, capacity: 10.0, length_km: 1.0 },
            ],
            od_pairs: vec![ODPair {
                origin: 0,
                destination: 3,
                demand: 6.0,
            }],
        }
    }

    #[test]
    fn test_frank_wolfe_converges() {
        let network = make_braess_network();
        let graph = Graph::from_network(&network);
        let config = FrankWolfeConfig {
            max_iterations: 100,
            convergence_threshold: 0.01,
            assignment_type: AssignmentType::UserEquilibrium,
        };

        let result = solve(&graph, &network.od_pairs, &config);

        // Should converge
        assert!(result.converged || result.relative_gap < 0.05);
        // Total demand should be conserved (6 vehicles entering, 6 leaving)
        // Flows on edges 0,1 (from source 0) should sum to 6
        let source_flow = result.edge_flows[0] + result.edge_flows[1];
        assert!((source_flow - 6.0).abs() < 0.1);
    }

    #[test]
    fn test_system_optimal_less_total_time() {
        let network = make_braess_network();
        let graph = Graph::from_network(&network);

        let ue_config = FrankWolfeConfig {
            max_iterations: 100,
            convergence_threshold: 0.001,
            assignment_type: AssignmentType::UserEquilibrium,
        };
        let so_config = FrankWolfeConfig {
            max_iterations: 100,
            convergence_threshold: 0.001,
            assignment_type: AssignmentType::SystemOptimal,
        };

        let ue_result = solve(&graph, &network.od_pairs, &ue_config);
        let so_result = solve(&graph, &network.od_pairs, &so_config);

        // System Optimal should have less or equal total system travel time
        assert!(so_result.total_system_travel_time <= ue_result.total_system_travel_time + 1.0);
    }

    #[test]
    fn test_single_od_pair_simple() {
        // Simple 2-node graph
        let network = RoadNetwork {
            nodes: vec![
                Node { id: 0, lat: 0.0, lon: 0.0 },
                Node { id: 1, lat: 1.0, lon: 0.0 },
            ],
            edges: vec![Edge {
                id: 0,
                source: 0,
                target: 1,
                free_flow_time: 10.0,
                capacity: 100.0,
                length_km: 1.0,
            }],
            od_pairs: vec![ODPair {
                origin: 0,
                destination: 1,
                demand: 50.0,
            }],
        };

        let graph = Graph::from_network(&network);
        let config = FrankWolfeConfig::default();
        let result = solve(&graph, &network.od_pairs, &config);

        // All flow should be on the single edge
        assert!((result.edge_flows[0] - 50.0).abs() < 1e-6);
    }
}
