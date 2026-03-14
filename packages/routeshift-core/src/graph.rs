use crate::types::{Edge, RoadNetwork};

/// Adjacency list representation of a directed road network graph.
///
/// Each entry in `adjacency[node_id]` is a list of (target_node, edge_id) tuples.
#[derive(Debug, Clone)]
pub struct Graph {
    pub num_nodes: usize,
    pub num_edges: usize,
    /// adjacency[source] = vec![(target, edge_id), ...]
    adjacency: Vec<Vec<(u32, u32)>>,
    /// Flat edge data indexed by edge_id
    pub edges: Vec<Edge>,
}

impl Graph {
    /// Build a graph from a RoadNetwork.
    ///
    /// Assumes node IDs are contiguous 0..num_nodes and edge IDs are contiguous 0..num_edges.
    pub fn from_network(network: &RoadNetwork) -> Self {
        let num_nodes = network.nodes.len();
        let num_edges = network.edges.len();
        let mut adjacency = vec![Vec::new(); num_nodes];

        for edge in &network.edges {
            adjacency[edge.source as usize].push((edge.target, edge.id));
        }

        Graph {
            num_nodes,
            num_edges,
            adjacency,
            edges: network.edges.clone(),
        }
    }

    /// Get outgoing neighbors and edge IDs for a node.
    #[inline]
    pub fn neighbors(&self, node: u32) -> &[(u32, u32)] {
        &self.adjacency[node as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Node;

    fn make_test_network() -> RoadNetwork {
        // Simple 4-node diamond graph:
        //   0 -> 1 -> 3
        //   0 -> 2 -> 3
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
    fn test_graph_construction() {
        let network = make_test_network();
        let graph = Graph::from_network(&network);
        assert_eq!(graph.num_nodes, 4);
        assert_eq!(graph.num_edges, 4);
    }

    #[test]
    fn test_neighbors() {
        let network = make_test_network();
        let graph = Graph::from_network(&network);

        let n0 = graph.neighbors(0);
        assert_eq!(n0.len(), 2);
        assert!(n0.contains(&(1, 0)));
        assert!(n0.contains(&(2, 1)));

        let n3 = graph.neighbors(3);
        assert_eq!(n3.len(), 0);
    }
}
