declare module "routeshift-wasm" {
  export default function init(): Promise<void>;

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
}
