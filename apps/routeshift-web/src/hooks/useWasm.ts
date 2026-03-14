"use client";

import { useState, useEffect, useRef } from "react";

interface WasmModule {
  solve_traffic_assignment: (network: unknown, config: unknown) => unknown;
  compute_shortest_path: (
    network: unknown,
    source: number,
    target: number
  ) => unknown;
  version: () => string;
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
