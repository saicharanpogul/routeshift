"use client";

import { useGameStore } from "@/stores/gameStore";

export function GameHUD() {
  const speed = useGameStore((s) => s.playerSpeed);
  const eta = useGameStore((s) => s.playerEta);
  const progress = useGameStore((s) => s.playerProgress);
  const tokens = useGameStore((s) => s.tokens);
  const totalTrips = useGameStore((s) => s.totalTrips);
  const numCars = useGameStore((s) => s.numCars);
  const timeScale = useGameStore((s) => s.timeScale);
  const setTimeScale = useGameStore((s) => s.setTimeScale);
  const complianceHistory = useGameStore((s) => s.complianceHistory);

  const complianceRate =
    complianceHistory.length > 0
      ? complianceHistory.filter(Boolean).length / complianceHistory.length
      : 0;

  const etaMin = Math.floor(eta / 60);
  const etaSec = Math.floor(eta % 60);

  return (
    <div className="absolute bottom-0 left-0 right-0 z-10">
      {/* Progress bar */}
      <div className="h-1 bg-white/10">
        <div
          className="h-full bg-amber-400 transition-all duration-300"
          style={{ width: `${progress * 100}%` }}
        />
      </div>

      <div className="bg-black/85 backdrop-blur-md border-t border-white/10 px-4 py-3">
        <div className="flex items-center justify-between max-w-5xl mx-auto">
          {/* Speed & ETA */}
          <div className="flex items-center gap-6">
            <div>
              <div className="text-2xl font-mono font-bold text-white">
                {speed.toFixed(0)}
              </div>
              <div className="text-[10px] text-white/40 uppercase">km/h</div>
            </div>
            <div>
              <div className="text-lg font-mono text-white">
                {etaMin}:{etaSec.toString().padStart(2, "0")}
              </div>
              <div className="text-[10px] text-white/40 uppercase">ETA</div>
            </div>
            <div>
              <div className="text-lg font-mono text-white">
                {(progress * 100).toFixed(0)}%
              </div>
              <div className="text-[10px] text-white/40 uppercase">
                Progress
              </div>
            </div>
          </div>

          {/* Token & stats */}
          <div className="flex items-center gap-6">
            <div className="text-center">
              <div className="text-lg font-mono text-amber-400 font-bold">
                {tokens}
              </div>
              <div className="text-[10px] text-white/40 uppercase">
                ROUTE Tokens
              </div>
            </div>
            <div className="text-center">
              <div className="text-sm font-mono text-white">{totalTrips}</div>
              <div className="text-[10px] text-white/40 uppercase">Trips</div>
            </div>
            <div className="text-center">
              <div className="text-sm font-mono text-white">
                {(complianceRate * 100).toFixed(0)}%
              </div>
              <div className="text-[10px] text-white/40 uppercase">
                Compliance
              </div>
            </div>
            <div className="text-center">
              <div className="text-sm font-mono text-white/60">{numCars}</div>
              <div className="text-[10px] text-white/40 uppercase">Cars</div>
            </div>
          </div>

          {/* Time scale */}
          <div className="flex items-center gap-1">
            {[1, 5, 10, 20].map((s) => (
              <button
                key={s}
                onClick={() => setTimeScale(s)}
                className={`px-2 py-1 text-xs rounded transition-colors ${
                  timeScale === s
                    ? "bg-white text-black font-medium"
                    : "bg-white/10 text-white/60 hover:bg-white/20"
                }`}
              >
                {s}x
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
