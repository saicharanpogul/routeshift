"use client";

import { RouteOption } from "@/types/game";

interface RouteChoicePanelProps {
  options: RouteOption[];
  onSelect: (index: number) => void;
  onHover: (index: number | null) => void;
}

const ROUTE_LABELS: Record<string, { label: string; color: string; desc: string }> = {
  Selfish: {
    label: "Fastest",
    color: "bg-blue-500",
    desc: "Fastest for you",
  },
  SystemSuggested: {
    label: "Suggested",
    color: "bg-emerald-500",
    desc: "Best for everyone",
  },
  Alternative: {
    label: "Alternative",
    color: "bg-purple-500",
    desc: "Different route",
  },
};

export function RouteChoicePanel({
  options,
  onSelect,
  onHover,
}: RouteChoicePanelProps) {
  return (
    <div className="absolute right-4 top-4 z-10 w-72 flex flex-col gap-2">
      <div className="bg-black/85 backdrop-blur-md rounded-lg p-4 border border-white/10">
        <h2 className="text-sm font-semibold text-white mb-1">Choose Your Route</h2>
        <p className="text-[10px] text-white/40 mb-3">
          Higher token rewards for routes that help reduce congestion
        </p>

        <div className="flex flex-col gap-2">
          {options.map((option, i) => {
            const info = ROUTE_LABELS[option.route_type] ?? ROUTE_LABELS.Alternative;
            const isSuggested = option.route_type === "SystemSuggested";

            return (
              <button
                key={i}
                onClick={() => onSelect(i)}
                onMouseEnter={() => onHover(i)}
                onMouseLeave={() => onHover(null)}
                className={`w-full text-left p-3 rounded-lg border transition-all ${
                  isSuggested
                    ? "border-emerald-500/50 bg-emerald-500/10 hover:bg-emerald-500/20"
                    : "border-white/10 bg-white/5 hover:bg-white/10"
                }`}
              >
                <div className="flex items-center justify-between mb-1.5">
                  <span
                    className={`text-[10px] font-medium px-2 py-0.5 rounded-full text-white ${info.color}`}
                  >
                    {info.label}
                  </span>
                  <span className="text-amber-400 font-mono text-sm font-bold">
                    +{option.token_reward}
                    <span className="text-[10px] text-amber-400/60 ml-0.5">
                      ROUTE
                    </span>
                  </span>
                </div>

                <div className="flex items-center gap-3 text-xs text-white/70">
                  <span>{option.estimated_time_minutes.toFixed(1)} min</span>
                  <span className="text-white/30">|</span>
                  <span>{option.total_distance_km.toFixed(1)} km</span>
                </div>

                <p className="text-[10px] text-white/40 mt-1">{info.desc}</p>
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
