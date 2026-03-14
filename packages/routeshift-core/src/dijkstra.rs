use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::graph::Graph;

/// State for Dijkstra's priority queue.
/// Uses reverse ordering so BinaryHeap acts as a min-heap.
#[derive(Debug, Clone)]
struct State {
    cost: f64,
    node: u32,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.node == other.node
    }
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.node.cmp(&other.node))
    }
}

/// Run Dijkstra's algorithm from a single source node.
///
/// Returns (distances, predecessors) where:
/// - distances[node] = shortest distance from source to node
/// - predecessors[node] = Some(edge_id) of the edge used to reach node, or None if unreachable
pub fn shortest_path_tree(
    graph: &Graph,
    source: u32,
    edge_costs: &[f64],
) -> (Vec<f64>, Vec<Option<u32>>) {
    let n = graph.num_nodes;
    let mut dist = vec![f64::MAX; n];
    let mut pred: Vec<Option<u32>> = vec![None; n];
    let mut heap = BinaryHeap::new();

    dist[source as usize] = 0.0;
    heap.push(State {
        cost: 0.0,
        node: source,
    });

    while let Some(State { cost, node }) = heap.pop() {
        if cost > dist[node as usize] {
            continue;
        }

        for &(target, edge_id) in graph.neighbors(node) {
            let edge_cost = edge_costs[edge_id as usize];
            let new_cost = cost + edge_cost;

            if new_cost < dist[target as usize] {
                dist[target as usize] = new_cost;
                pred[target as usize] = Some(edge_id);
                heap.push(State {
                    cost: new_cost,
                    node: target,
                });
            }
        }
    }

    (dist, pred)
}

/// Reconstruct the path (as edge IDs) from source to target using predecessor array.
///
/// Returns None if target is unreachable.
pub fn reconstruct_path(
    graph: &Graph,
    predecessors: &[Option<u32>],
    target: u32,
) -> Option<Vec<u32>> {
    if predecessors[target as usize].is_none() {
        return None;
    }

    let mut path_edges = Vec::new();
    let mut current = target;

    while let Some(edge_id) = predecessors[current as usize] {
        path_edges.push(edge_id);
        current = graph.edges[edge_id as usize].source;
    }

    path_edges.reverse();
    Some(path_edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Edge, Node, RoadNetwork};

    fn make_test_network() -> RoadNetwork {
        // Diamond: 0->1->3, 0->2->3
        // Costs: 0->1: 5, 1->3: 10, 0->2: 10, 2->3: 5
        // Shortest 0->3: via either path = 15
        RoadNetwork {
            nodes: vec![
                Node { id: 0, lat: 0.0, lon: 0.0 },
                Node { id: 1, lat: 1.0, lon: 0.0 },
                Node { id: 2, lat: 0.0, lon: 1.0 },
                Node { id: 3, lat: 1.0, lon: 1.0 },
            ],
            edges: vec![
                Edge { id: 0, source: 0, target: 1, free_flow_time: 5.0, capacity: 100.0, length_km: 1.0 },
                Edge { id: 1, source: 0, target: 2, free_flow_time: 10.0, capacity: 200.0, length_km: 2.0 },
                Edge { id: 2, source: 1, target: 3, free_flow_time: 10.0, capacity: 200.0, length_km: 2.0 },
                Edge { id: 3, source: 2, target: 3, free_flow_time: 5.0, capacity: 100.0, length_km: 1.0 },
            ],
            od_pairs: vec![],
        }
    }

    #[test]
    fn test_shortest_path_diamond() {
        let network = make_test_network();
        let graph = Graph::from_network(&network);
        let costs: Vec<f64> = graph.edges.iter().map(|e| e.free_flow_time).collect();

        let (dist, pred) = shortest_path_tree(&graph, 0, &costs);

        assert!((dist[0] - 0.0).abs() < 1e-10);
        // Both paths cost 15, either is valid
        assert!((dist[3] - 15.0).abs() < 1e-10);

        let path = reconstruct_path(&graph, &pred, 3).unwrap();
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn test_unreachable_node() {
        let network = make_test_network();
        let graph = Graph::from_network(&network);
        let costs: Vec<f64> = graph.edges.iter().map(|e| e.free_flow_time).collect();

        // Node 0 has no incoming edges, so it's unreachable from node 3
        let (dist, pred) = shortest_path_tree(&graph, 3, &costs);
        assert_eq!(dist[0], f64::MAX);
        assert!(reconstruct_path(&graph, &pred, 0).is_none());
    }

    #[test]
    fn test_source_to_self() {
        let network = make_test_network();
        let graph = Graph::from_network(&network);
        let costs: Vec<f64> = graph.edges.iter().map(|e| e.free_flow_time).collect();

        let (dist, _) = shortest_path_tree(&graph, 0, &costs);
        assert!((dist[0] - 0.0).abs() < 1e-10);
    }
}
