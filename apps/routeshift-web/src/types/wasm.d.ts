declare module "routeshift-wasm" {
  export default function init(): Promise<void>;

  // Original API
  export function solve_traffic_assignment(
    network: unknown,
    config: unknown
  ): unknown;
  export function compute_shortest_path(
    network: unknown,
    source: number,
    target: number
  ): unknown;
  export function version(): string;

  // Simulation API
  export function init_simulation(
    network: unknown,
    config: unknown,
    edge_geometries: unknown
  ): void;
  export function sim_tick(dt_seconds: number): unknown;
  export function spawn_player(origin: number): void;
  export function compute_route_options(
    origin: number,
    destination: number
  ): unknown;
  export function set_player_route(route_index: number): void;
  export function get_route_reward(route_index: number): number;
  export function find_nearest_node(lng: number, lat: number): number;
}
