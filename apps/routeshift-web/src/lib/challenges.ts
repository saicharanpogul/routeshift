import { Challenge } from "@/types/game";

const CHALLENGE_TEMPLATES = [
  {
    id: "rush_hour",
    title: "Rush Hour Survivor",
    description: "Complete a trip under 20 minutes during heavy traffic",
    type: "time_trial" as const,
    target: 20,
    reward: 50,
  },
  {
    id: "system_hero_3",
    title: "System Hero",
    description: "Follow 3 system-suggested routes in a row",
    type: "compliance_streak" as const,
    target: 3,
    reward: 75,
  },
  {
    id: "system_hero_5",
    title: "Super Cooperator",
    description: "Follow 5 system-suggested routes in a row",
    type: "compliance_streak" as const,
    target: 5,
    reward: 150,
  },
  {
    id: "efficiency_10",
    title: "Efficiency Expert",
    description: "Complete 10 trips total",
    type: "efficiency" as const,
    target: 10,
    reward: 200,
  },
  {
    id: "token_collector",
    title: "Token Collector",
    description: "Earn 500 ROUTE tokens total",
    type: "efficiency" as const,
    target: 500,
    reward: 100,
  },
];

export function getNextChallenge(completedIds: string[]): Challenge | null {
  const available = CHALLENGE_TEMPLATES.filter(
    (t) => !completedIds.includes(t.id)
  );
  if (available.length === 0) return null;
  return { ...available[0], progress: 0 };
}

export function updateChallengeProgress(
  challenge: Challenge,
  stats: {
    lastTripTimeMinutes: number;
    complianceStreak: number;
    totalTrips: number;
    totalTokens: number;
  }
): Challenge {
  const updated = { ...challenge };

  switch (challenge.type) {
    case "time_trial":
      if (stats.lastTripTimeMinutes <= challenge.target) {
        updated.progress = challenge.target;
      }
      break;
    case "compliance_streak":
      updated.progress = Math.min(stats.complianceStreak, challenge.target);
      break;
    case "efficiency":
      if (challenge.id.includes("token")) {
        updated.progress = Math.min(stats.totalTokens, challenge.target);
      } else {
        updated.progress = Math.min(stats.totalTrips, challenge.target);
      }
      break;
  }

  return updated;
}

export function isChallengeComplete(challenge: Challenge): boolean {
  return challenge.progress >= challenge.target;
}
