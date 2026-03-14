"use client";

import { useState } from "react";
import { useGameStore } from "@/stores/gameStore";
import { getLeaderboard } from "@/lib/leaderboard";

export function Leaderboard() {
  const [open, setOpen] = useState(false);
  const tokens = useGameStore((s) => s.tokens);
  const trips = useGameStore((s) => s.totalTrips);
  const history = useGameStore((s) => s.complianceHistory);

  const rate = history.length > 0 ? history.filter(Boolean).length / history.length : 0;
  const entries = getLeaderboard(tokens, trips, rate);

  return (
    <>
      <button
        onClick={() => setOpen(!open)}
        className="absolute top-4 right-4 z-10 bg-black/80 backdrop-blur-md rounded-lg px-3 py-2 border border-white/10 text-xs text-white/70 hover:text-white transition-colors"
      >
        Leaderboard
      </button>

      {open && (
        <div className="absolute top-14 right-4 z-10 w-72 bg-black/90 backdrop-blur-md rounded-lg border border-white/10 p-4">
          <h3 className="text-sm font-semibold text-white mb-3">Leaderboard</h3>

          <div className="space-y-1">
            {entries.map((entry, i) => (
              <div
                key={entry.name}
                className={`flex items-center justify-between py-1.5 px-2 rounded text-xs ${
                  entry.isPlayer
                    ? "bg-amber-500/10 border border-amber-500/30"
                    : ""
                }`}
              >
                <div className="flex items-center gap-2">
                  <span className="text-white/40 w-4 text-right font-mono">
                    {i + 1}
                  </span>
                  <span
                    className={
                      entry.isPlayer
                        ? "text-amber-400 font-medium"
                        : "text-white/70"
                    }
                  >
                    {entry.name}
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-amber-400 font-mono">
                    {entry.tokens}
                  </span>
                  <span className="text-white/30 w-8 text-right">
                    {(entry.complianceRate * 100).toFixed(0)}%
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </>
  );
}
