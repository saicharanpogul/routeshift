"use client";

import { useEffect, useRef } from "react";
import { useGameStore } from "@/stores/gameStore";
import { SimSnapshot } from "@/types/game";
import { WasmModule } from "./useWasm";

export function useSimulationLoop(
  wasm: WasmModule | null,
  running: boolean
) {
  const frameRef = useRef<number>(0);
  const lastTimeRef = useRef<number>(0);

  useEffect(() => {
    if (!wasm || !running) {
      lastTimeRef.current = 0;
      return;
    }

    const loop = (timestamp: number) => {
      const dt = lastTimeRef.current
        ? (timestamp - lastTimeRef.current) / 1000
        : 1 / 60;
      lastTimeRef.current = timestamp;

      const cappedDt = Math.min(dt, 0.1);

      try {
        const snapshot = wasm.sim_tick(cappedDt) as SimSnapshot;
        useGameStore.getState().updateSimFrame(snapshot);

        // Check if player arrived
        if (
          snapshot.player_arrived &&
          useGameStore.getState().mode === "driving"
        ) {
          const state = useGameStore.getState();
          const route = state.routeOptions?.[state.selectedRouteIndex ?? 0];
          const tokensEarned = route?.token_reward ?? 10;
          const routeType = route?.route_type ?? "Selfish";
          state.playerArrived(tokensEarned, routeType);
        }
      } catch {
        // Sim tick can fail during re-initialization
      }

      frameRef.current = requestAnimationFrame(loop);
    };

    frameRef.current = requestAnimationFrame(loop);

    return () => {
      if (frameRef.current) cancelAnimationFrame(frameRef.current);
    };
  }, [wasm, running]);
}
