"use client";

import { TripResult } from "@/types/game";

interface TripSummaryProps {
  trip: TripResult;
  totalTokens: number;
  onNextTrip: () => void;
}

export function TripSummary({ trip, totalTokens, onNextTrip }: TripSummaryProps) {
  return (
    <div className="absolute inset-0 z-20 flex items-center justify-center bg-black/40 backdrop-blur-sm">
      <div className="bg-black/90 backdrop-blur-md rounded-xl p-6 border border-white/10 w-80 shadow-2xl">
        <div className="text-center mb-4">
          <div className="text-3xl mb-1">
            {trip.wasCompliant ? "\u2728" : "\u{1F3C1}"}
          </div>
          <h2 className="text-lg font-semibold text-white">Trip Complete!</h2>
        </div>

        {/* Token reward */}
        <div className="text-center mb-4 py-3 bg-amber-500/10 rounded-lg border border-amber-500/20">
          <div className="text-3xl font-bold text-amber-400 font-mono">
            +{trip.tokensEarned}
          </div>
          <div className="text-xs text-amber-400/60 uppercase">
            ROUTE Tokens Earned
          </div>
        </div>

        {/* Trip stats */}
        <div className="grid grid-cols-2 gap-2 mb-4">
          <StatBlock label="Time" value={`${trip.timeMinutes.toFixed(1)} min`} />
          <StatBlock
            label="Distance"
            value={`${trip.distanceKm.toFixed(1)} km`}
          />
          <StatBlock
            label="Avg Speed"
            value={`${trip.avgSpeedKmh.toFixed(0)} km/h`}
          />
          <StatBlock
            label="Route Type"
            value={
              trip.routeType === "SystemSuggested"
                ? "Suggested"
                : trip.routeType === "Selfish"
                  ? "Fastest"
                  : "Alternative"
            }
          />
        </div>

        {/* Compliance message */}
        {trip.wasCompliant && (
          <div className="text-center text-xs text-emerald-400 mb-4 p-2 bg-emerald-500/10 rounded-md">
            You followed the system-suggested route and helped reduce
            congestion for everyone!
          </div>
        )}

        {/* Total tokens */}
        <div className="text-center text-xs text-white/40 mb-4">
          Total tokens: <span className="text-amber-400">{totalTokens}</span>
        </div>

        <button
          onClick={onNextTrip}
          className="w-full py-2.5 rounded-lg bg-white text-black font-medium text-sm hover:bg-white/90 transition-colors"
        >
          Next Trip
        </button>
      </div>
    </div>
  );
}

function StatBlock({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-white/5 rounded-md p-2">
      <div className="text-[10px] text-white/40 uppercase">{label}</div>
      <div className="text-sm font-mono text-white">{value}</div>
    </div>
  );
}
