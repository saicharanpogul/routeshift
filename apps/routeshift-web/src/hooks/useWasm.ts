"use client";

import { useState, useEffect, useRef } from "react";

export interface WasmModule {
  // Original API
  solve_traffic_assignment: (network: unknown, config: unknown) => unknown;
  compute_shortest_path: (network: unknown, source: number, target: number) => unknown;
  version: () => string;
  // Simulation API
  init_simulation: (network: unknown, config: unknown, geometries: unknown) => void;
  sim_tick: (dt: number) => unknown;
  spawn_player: (origin: number) => void;
  compute_route_options: (origin: number, destination: number) => unknown;
  set_player_route: (routeIndex: number) => void;
  get_route_reward: (routeIndex: number) => number;
  find_nearest_node: (lng: number, lat: number) => number;
}

export function useWasm() {
  const [wasm, setWasm] = useState<WasmModule | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const initRef = useRef(false);

  useEffect(() => {
    if (initRef.current) return;
    initRef.current = true;

    async function loadWasm() {
      try {
        const wasmModule = await import("routeshift-wasm");
        await wasmModule.default();
        setWasm(wasmModule as unknown as WasmModule);
      } catch (err) {
        console.error("Failed to load WASM module:", err);
        setError(err instanceof Error ? err.message : "Failed to load WASM");
      } finally {
        setLoading(false);
      }
    }

    loadWasm();
  }, []);

  return { wasm, loading, error };
}
