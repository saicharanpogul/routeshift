export interface Node {
  id: number;
  lat: number;
  lon: number;
}

export interface Edge {
  id: number;
  source: number;
  target: number;
  free_flow_time: number;
  capacity: number;
  length_km: number;
}

export interface ODPair {
  origin: number;
  destination: number;
  demand: number;
}

export interface RoadNetwork {
  nodes: Node[];
  edges: Edge[];
  od_pairs: ODPair[];
}

export type AssignmentType = "UserEquilibrium" | "SystemOptimal";

export interface FrankWolfeConfig {
  max_iterations: number;
  convergence_threshold: number;
  assignment_type: AssignmentType;
}

export interface AssignmentResult {
  edge_flows: number[];
  edge_travel_times: number[];
  total_system_travel_time: number;
  iterations: number;
  converged: boolean;
  relative_gap: number;
}

export interface CityConfig {
  name: string;
  label: string;
  center: [number, number]; // [lat, lon]
  zoom: number;
}
