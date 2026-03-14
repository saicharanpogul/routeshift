"use client";

import { CityConfig } from "@/types/graph";
import { CitySelector } from "./CitySelector";
import { GameMode } from "@/types/game";

interface GameControlPanelProps {
  city: CityConfig;
  mode: GameMode;
  tokens: number;
  simRunning: boolean;
  onCityChange: (city: CityConfig) => void;
  onStart: () => void;
}

export function GameControlPanel({
  city,
  mode,
  tokens,
  simRunning,
  onCityChange,
  onStart,
}: GameControlPanelProps) {
  return (
    <div className="bg-black/85 backdrop-blur-md rounded-lg p-4 border border-white/10">
        <div className="flex items-center justify-between mb-1">
          <h1 className="text-lg font-semibold text-white">RouteShift</h1>
          <div className="flex items-center gap-1.5">
            <span className="text-amber-400 font-mono text-sm font-bold">
              {tokens}
            </span>
            <span className="text-[10px] text-amber-400/60">ROUTE</span>
          </div>
        </div>
        <p className="text-[10px] text-white/40 mb-3">
          {mode === "idle"
            ? "Choose a city and start driving"
            : mode === "choosing_destination"
              ? "Click anywhere on the map to set your destination"
              : mode === "choosing_route"
                ? "Pick a route to start driving"
                : mode === "driving"
                  ? "You are driving... watch the traffic!"
                  : "Trip complete!"}
        </p>

        <div className="mb-3">
          <CitySelector selected={city} onSelect={onCityChange} />
        </div>

        {!simRunning && (
          <button
            onClick={onStart}
            className="w-full py-2.5 rounded-lg bg-white text-black font-medium text-sm hover:bg-white/90 transition-colors"
          >
            Start Driving
          </button>
        )}

        {mode === "choosing_destination" && (
          <div className="mt-2 text-center">
            <div className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-blue-500/10 border border-blue-500/30 rounded-full">
              <div className="w-2 h-2 bg-blue-400 rounded-full animate-pulse" />
              <span className="text-[11px] text-blue-400">
                Click the map to set destination
              </span>
            </div>
          </div>
        )}
    </div>
  );
}
