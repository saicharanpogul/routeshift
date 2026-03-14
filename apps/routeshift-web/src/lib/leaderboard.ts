import { LeaderboardEntry } from "@/types/game";

const SEEDED_PLAYERS: LeaderboardEntry[] = [
  { name: "TrafficMaster", tokens: 1250, trips: 45, complianceRate: 0.92 },
  { name: "RouteRunner", tokens: 980, trips: 38, complianceRate: 0.85 },
  { name: "CityNavigator", tokens: 870, trips: 42, complianceRate: 0.78 },
  { name: "GreenCommuter", tokens: 750, trips: 30, complianceRate: 0.95 },
  { name: "SpeedDemon", tokens: 620, trips: 55, complianceRate: 0.45 },
  { name: "FlowOptimizer", tokens: 540, trips: 25, complianceRate: 0.88 },
  { name: "RoadWarrior", tokens: 480, trips: 35, complianceRate: 0.6 },
  { name: "SmartDriver", tokens: 350, trips: 20, complianceRate: 0.82 },
  { name: "PathFinder", tokens: 280, trips: 18, complianceRate: 0.72 },
  { name: "CasualCruiser", tokens: 150, trips: 12, complianceRate: 0.5 },
];

export function getLeaderboard(
  playerTokens: number,
  playerTrips: number,
  playerComplianceRate: number
): (LeaderboardEntry & { isPlayer: boolean })[] {
  const playerEntry: LeaderboardEntry = {
    name: "You",
    tokens: playerTokens,
    trips: playerTrips,
    complianceRate: playerComplianceRate,
  };

  const all = [
    ...SEEDED_PLAYERS.map((e) => ({ ...e, isPlayer: false })),
    { ...playerEntry, isPlayer: true },
  ];

  all.sort((a, b) => b.tokens - a.tokens);
  return all.slice(0, 10);
}
