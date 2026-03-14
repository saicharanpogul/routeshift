"use client";

import { useState, useCallback, useEffect } from "react";
import { MapView } from "@/components/MapView";
import { GameControlPanel } from "@/components/GameControlPanel";
import { GameHUD } from "@/components/GameHUD";
import { RouteChoicePanel } from "@/components/RouteChoicePanel";
import { TripSummary } from "@/components/TripSummary";
import { Leaderboard } from "@/components/Leaderboard";
import { ChallengePanel } from "@/components/ChallengePanel";
import { useWasm } from "@/hooks/useWasm";
import { useMapData } from "@/hooks/useMapData";
import { useSimulationLoop } from "@/hooks/useSimulationLoop";
import { useGameStore } from "@/stores/gameStore";
import { extractEdgeGeometries } from "@/lib/geometryUtils";
import {
  getNextChallenge,
  updateChallengeProgress,
  isChallengeComplete,
} from "@/lib/challenges";
import { CityConfig } from "@/types/graph";
import { RouteOption } from "@/types/game";

export default function Home() {
  const { wasm, loading: wasmLoading } = useWasm();
  const store = useGameStore();
  const { geojson, graph, loading: dataLoading } = useMapData(store.city.name);
  const [hoveredRouteIndex, setHoveredRouteIndex] = useState<number | null>(null);
  const [simInitialized, setSimInitialized] = useState(false);

  // Run the simulation loop
  useSimulationLoop(wasm, store.simRunning);

  // Initialize simulation when city data loads
  useEffect(() => {
    if (!wasm || !graph || !geojson || simInitialized) return;

    try {
      const geometries = extractEdgeGeometries(geojson);
      wasm.init_simulation(
        graph,
        { num_ai_agents: 120, ai_compliance_rate: 0.6, time_scale: store.timeScale },
        geometries
      );
      setSimInitialized(true);
    } catch (e) {
      console.error("Failed to init simulation:", e);
    }
  }, [wasm, graph, geojson, simInitialized, store.timeScale]);

  // Re-initialize on city change
  useEffect(() => {
    setSimInitialized(false);
  }, [store.city.name]);

  // Handle city change
  const handleCityChange = useCallback(
    (city: CityConfig) => {
      store.setCity(city);
      setSimInitialized(false);
    },
    [store]
  );

  // Start game
  const handleStart = useCallback(() => {
    if (!simInitialized) return;
    store.startSimulation();

    // Start a challenge if none active
    if (!store.activeChallenge) {
      const challenge = getNextChallenge(store.completedChallenges);
      if (challenge) store.setChallenge(challenge);
    }
  }, [simInitialized, store]);

  // Handle map click for destination
  const handleMapClick = useCallback(
    (lngLat: [number, number]) => {
      if (store.mode !== "choosing_destination" || !wasm) return;

      // Find nearest node
      const destNode = wasm.find_nearest_node(lngLat[0], lngLat[1]);

      // Pick a random origin (use the nearest node to city center as origin)
      const originNode = wasm.find_nearest_node(
        store.city.center[1],
        store.city.center[0]
      );

      if (destNode === originNode) return;

      // Spawn player and compute routes
      wasm.spawn_player(originNode);
      store.setPlayerOrigin(originNode);
      store.setPlayerDestination(destNode);

      const options = wasm.compute_route_options(
        originNode,
        destNode
      ) as RouteOption[];

      if (options.length > 0) {
        store.setRouteOptions(options);
      }
    },
    [store, wasm]
  );

  // Handle route selection
  const handleSelectRoute = useCallback(
    (index: number) => {
      if (!wasm) return;
      wasm.set_player_route(index);
      store.selectRoute(index);
      setHoveredRouteIndex(null);
    },
    [wasm, store]
  );

  // Handle next trip
  const handleNextTrip = useCallback(() => {
    // Update challenge progress
    if (store.activeChallenge && store.lastTrip) {
      const updated = updateChallengeProgress(store.activeChallenge, {
        lastTripTimeMinutes: store.lastTrip.timeMinutes,
        complianceStreak: store.complianceStreak,
        totalTrips: store.totalTrips,
        totalTokens: store.tokens,
      });

      if (isChallengeComplete(updated)) {
        store.completeChallenge(updated.id);
        // Get next challenge
        const next = getNextChallenge([
          ...store.completedChallenges,
          updated.id,
        ]);
        store.setChallenge(next);
      } else {
        store.setChallenge(updated);
      }
    }

    store.resetTrip();
  }, [store]);

  const isLoading = wasmLoading || dataLoading;

  return (
    <main className="relative w-screen h-screen">
      <MapView
        city={store.city}
        geojson={geojson}
        onMapClick={handleMapClick}
        routeOptions={store.mode === "choosing_route" ? store.routeOptions : null}
        hoveredRouteIndex={hoveredRouteIndex}
        clickable={store.mode === "choosing_destination"}
      />

      {/* Loading overlay */}
      {isLoading && (
        <div className="absolute inset-0 z-30 flex items-center justify-center bg-black/60 backdrop-blur-sm">
          <div className="text-white text-sm">Loading...</div>
        </div>
      )}

      {/* Game control panel (top-left) */}
      <GameControlPanel
        city={store.city}
        mode={store.mode}
        tokens={store.tokens}
        simRunning={store.simRunning}
        onCityChange={handleCityChange}
        onStart={handleStart}
      />

      {/* Challenge panel */}
      {store.simRunning && <ChallengePanel challenge={store.activeChallenge} />}

      {/* Route choice panel (right side) */}
      {store.mode === "choosing_route" && store.routeOptions && (
        <RouteChoicePanel
          options={store.routeOptions}
          onSelect={handleSelectRoute}
          onHover={setHoveredRouteIndex}
        />
      )}

      {/* Driving HUD (bottom) */}
      {store.mode === "driving" && <GameHUD />}

      {/* Trip summary (center overlay) */}
      {store.mode === "arrived" && store.lastTrip && (
        <TripSummary
          trip={store.lastTrip}
          totalTokens={store.tokens}
          onNextTrip={handleNextTrip}
        />
      )}

      {/* Leaderboard (top-right) */}
      {store.simRunning && <Leaderboard />}

      {/* Congestion legend */}
      {store.simRunning && (
        <div className="absolute bottom-20 left-4 z-10 bg-black/80 backdrop-blur-md rounded-lg p-3 border border-white/10">
          <div className="text-[10px] text-white/50 uppercase tracking-wide mb-1.5">
            Congestion
          </div>
          <div className="flex items-center gap-1">
            <div className="w-20 h-2 rounded-full bg-gradient-to-r from-blue-500 via-green-500 via-yellow-500 to-red-700" />
            <div className="flex justify-between w-20 text-[9px] text-white/40">
              <span>Free</span>
              <span>Jammed</span>
            </div>
          </div>
          <div className="flex items-center gap-2 mt-2">
            <div className="w-3 h-3 rounded-full bg-blue-400 border border-blue-300/50" />
            <span className="text-[10px] text-white/50">AI Cars</span>
            <div className="w-3 h-3 rounded-full bg-amber-400 border-2 border-white" />
            <span className="text-[10px] text-white/50">Your Car</span>
          </div>
        </div>
      )}
    </main>
  );
}
