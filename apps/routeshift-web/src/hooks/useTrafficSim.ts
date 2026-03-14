"use client";

import { useState, useCallback } from "react";
import {
  AssignmentResult,
  AssignmentType,
  FrankWolfeConfig,
  RoadNetwork,
} from "@/types/graph";

interface TrafficSimState {
  result: AssignmentResult | null;
  running: boolean;
  error: string | null;
  config: FrankWolfeConfig;
}

interface WasmModule {
  solve_traffic_assignment: (network: unknown, config: unknown) => unknown;
}

export function useTrafficSim(
  wasm: WasmModule | null,
  graph: RoadNetwork | null
) {
  const [state, setState] = useState<TrafficSimState>({
    result: null,
    running: false,
    error: null,
    config: {
      max_iterations: 100,
      convergence_threshold: 0.01,
      assignment_type: "UserEquilibrium",
    },
  });

  const runAssignment = useCallback(() => {
    if (!wasm || !graph) return;

    setState((prev) => ({ ...prev, running: true, error: null }));

    try {
      const result = wasm.solve_traffic_assignment(
        graph,
        state.config
      ) as AssignmentResult;
      setState((prev) => ({ ...prev, result, running: false }));
    } catch (err) {
      setState((prev) => ({
        ...prev,
        running: false,
        error:
          err instanceof Error ? err.message : "Assignment failed",
      }));
    }
  }, [wasm, graph, state.config]);

  const setAssignmentType = useCallback((type: AssignmentType) => {
    setState((prev) => ({
      ...prev,
      config: { ...prev.config, assignment_type: type },
    }));
  }, []);

  const setMaxIterations = useCallback((n: number) => {
    setState((prev) => ({
      ...prev,
      config: { ...prev.config, max_iterations: n },
    }));
  }, []);

  return {
    ...state,
    runAssignment,
    setAssignmentType,
    setMaxIterations,
  };
}
