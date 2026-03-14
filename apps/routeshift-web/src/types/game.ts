export interface SimSnapshot {
  car_positions: number[];
  car_types: number[];
  edge_flows: number[];
  game_time: number;
  player_progress: number;
  player_speed_kmh: number;
  player_eta_seconds: number;
  player_arrived: boolean;
  num_cars: number;
}

export interface RouteOption {
  edge_ids: number[];
  total_distance_km: number;
  estimated_time_minutes: number;
  token_reward: number;
  route_type: "Selfish" | "SystemSuggested" | "Alternative";
  geometry: [number, number][];
}

export interface SimConfig {
  num_ai_agents: number;
  ai_compliance_rate: number;
  time_scale: number;
}

export type GameMode =
  | "idle"
  | "choosing_destination"
  | "choosing_route"
  | "driving"
  | "arrived";

export interface Challenge {
  id: string;
  title: string;
  description: string;
  type: "time_trial" | "compliance_streak" | "efficiency";
  target: number;
  reward: number;
  progress: number;
}

export interface LeaderboardEntry {
  name: string;
  tokens: number;
  trips: number;
  complianceRate: number;
}

export interface TripResult {
  timeMinutes: number;
  distanceKm: number;
  avgSpeedKmh: number;
  tokensEarned: number;
  routeType: string;
  wasCompliant: boolean;
}
