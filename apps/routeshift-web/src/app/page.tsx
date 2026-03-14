"use client";

import { useState } from "react";
import { MapView } from "@/components/MapView";
import { CitySelector } from "@/components/CitySelector";
import { ControlPanel } from "@/components/ControlPanel";
import { StatsPanel } from "@/components/StatsPanel";
import { useWasm } from "@/hooks/useWasm";
import { useMapData } from "@/hooks/useMapData";
import { useTrafficSim } from "@/hooks/useTrafficSim";
import { CITIES } from "@/lib/cities";
import { CityConfig } from "@/types/graph";

export default function Home() {
  const [city, setCity] = useState<CityConfig>(CITIES[0]);
  const { wasm, loading: wasmLoading, error: wasmError } = useWasm();
  const {
    geojson,
    graph,
    loading: dataLoading,
    error: dataError,
  } = useMapData(city.name);
  const {
    result,
    running,
    error: simError,
    config,
    runAssignment,
    setAssignmentType,
  } = useTrafficSim(wasm, graph);

  return (
    <main className="relative w-screen h-screen">
      <MapView city={city} geojson={geojson} result={result} />

      {/* Top-left panel */}
      <div className="absolute top-4 left-4 z-10 flex flex-col gap-3 w-72">
        {/* Header */}
        <div className="bg-black/80 backdrop-blur-md rounded-lg p-4 border border-white/10">
          <h1 className="text-lg font-semibold text-white">RouteShift</h1>
          <p className="text-xs text-white/50 mt-0.5">
            Incentive-based traffic redistribution
          </p>
          <div className="mt-3">
            <CitySelector selected={city} onSelect={setCity} />
          </div>
        </div>

        {/* Controls */}
        <div className="bg-black/80 backdrop-blur-md rounded-lg p-4 border border-white/10">
          <ControlPanel
            assignmentType={config.assignment_type}
            onAssignmentTypeChange={setAssignmentType}
            onRun={runAssignment}
            running={running}
            wasmReady={!!wasm}
            dataReady={!!graph}
          />
        </div>

        {/* Stats */}
        <div className="bg-black/80 backdrop-blur-md rounded-lg p-4 border border-white/10">
          <StatsPanel
            result={result}
            loading={dataLoading}
            wasmLoading={wasmLoading}
            wasmError={wasmError || simError}
            dataError={dataError}
          />
        </div>
      </div>

      {/* Legend */}
      {result && (
        <div className="absolute bottom-6 left-4 z-10 bg-black/80 backdrop-blur-md rounded-lg p-3 border border-white/10">
          <div className="text-[10px] text-white/50 uppercase tracking-wide mb-1.5">
            Congestion (V/C Ratio)
          </div>
          <div className="flex items-center gap-1">
            <div className="w-16 h-2 rounded-full bg-gradient-to-r from-green-500 via-yellow-500 to-red-700" />
            <div className="flex justify-between w-16 text-[9px] text-white/40">
              <span>0</span>
              <span>1+</span>
            </div>
          </div>
        </div>
      )}
    </main>
  );
}
