"use client";

import { Challenge } from "@/types/game";

interface ChallengePanelProps {
  challenge: Challenge | null;
}

export function ChallengePanel({ challenge }: ChallengePanelProps) {
  if (!challenge) return null;

  const progressPercent = Math.min(
    (challenge.progress / challenge.target) * 100,
    100
  );

  return (
    <div className="bg-black/80 backdrop-blur-md rounded-lg p-3 border border-white/10">
      <div className="flex items-center justify-between mb-1">
        <span className="text-[10px] text-purple-400 uppercase font-medium tracking-wide">
          Challenge
        </span>
        <span className="text-[10px] text-amber-400 font-mono">
          +{challenge.reward} ROUTE
        </span>
      </div>

      <h4 className="text-xs font-medium text-white mb-0.5">
        {challenge.title}
      </h4>
      <p className="text-[10px] text-white/40 mb-2">{challenge.description}</p>

      {/* Progress bar */}
      <div className="h-1.5 bg-white/10 rounded-full overflow-hidden">
        <div
          className="h-full bg-purple-500 transition-all duration-500 rounded-full"
          style={{ width: `${progressPercent}%` }}
        />
      </div>
      <div className="text-[10px] text-white/40 mt-1 text-right">
        {challenge.progress}/{challenge.target}
      </div>
    </div>
  );
}
