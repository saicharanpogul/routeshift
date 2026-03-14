import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
  GameMode,
  SimSnapshot,
  RouteOption,
  Challenge,
  TripResult,
} from "@/types/game";
import { CityConfig } from "@/types/graph";
import { CITIES } from "@/lib/cities";

interface GameState {
  // Game mode
  mode: GameMode;

  // City
  city: CityConfig;

  // Player state
  playerOrigin: number | null;
  playerDestination: number | null;
  playerPosition: [number, number] | null;
  playerSpeed: number;
  playerEta: number;
  playerProgress: number;

  // Route options
  routeOptions: RouteOption[] | null;
  selectedRouteIndex: number | null;

  // Economy
  tokens: number;
  totalTrips: number;
  complianceHistory: boolean[];
  complianceStreak: number;

  // Simulation
  simRunning: boolean;
  gameTime: number;
  timeScale: number;
  carPositions: number[] | null;
  carTypes: number[] | null;
  edgeFlows: number[] | null;
  numCars: number;

  // Challenges
  activeChallenge: Challenge | null;
  completedChallenges: string[];

  // Last trip
  lastTrip: TripResult | null;
  tripStartTime: number;

  // Actions
  setCity: (city: CityConfig) => void;
  setMode: (mode: GameMode) => void;
  startSimulation: () => void;
  setTimeScale: (scale: number) => void;
  setPlayerOrigin: (nodeId: number) => void;
  setPlayerDestination: (nodeId: number) => void;
  setRouteOptions: (options: RouteOption[]) => void;
  selectRoute: (index: number) => void;
  updateSimFrame: (snapshot: SimSnapshot) => void;
  playerArrived: (tokensEarned: number, routeType: string) => void;
  setChallenge: (challenge: Challenge | null) => void;
  completeChallenge: (id: string) => void;
  resetTrip: () => void;
}

export const useGameStore = create<GameState>()(
  persist(
    (set, get) => ({
      mode: "idle",
      city: CITIES[0],
      playerOrigin: null,
      playerDestination: null,
      playerPosition: null,
      playerSpeed: 0,
      playerEta: 0,
      playerProgress: 0,
      routeOptions: null,
      selectedRouteIndex: null,
      tokens: 0,
      totalTrips: 0,
      complianceHistory: [],
      complianceStreak: 0,
      simRunning: false,
      gameTime: 0,
      timeScale: 10,
      carPositions: null,
      carTypes: null,
      edgeFlows: null,
      numCars: 0,
      activeChallenge: null,
      completedChallenges: [],
      lastTrip: null,
      tripStartTime: 0,

      setCity: (city) => set({ city, mode: "idle", simRunning: false }),
      setMode: (mode) => set({ mode }),
      startSimulation: () => set({ simRunning: true, mode: "choosing_destination" }),
      setTimeScale: (scale) => set({ timeScale: scale }),
      setPlayerOrigin: (nodeId) => set({ playerOrigin: nodeId }),
      setPlayerDestination: (nodeId) => set({ playerDestination: nodeId }),
      setRouteOptions: (options) =>
        set({ routeOptions: options, mode: "choosing_route" }),
      selectRoute: (index) => {
        const state = get();
        const route = state.routeOptions?.[index];
        set({
          selectedRouteIndex: index,
          mode: "driving",
          routeOptions: state.routeOptions,
          tripStartTime: state.gameTime,
          lastTrip: null,
        });
        return route;
      },

      updateSimFrame: (snapshot) =>
        set({
          carPositions: snapshot.car_positions,
          carTypes: snapshot.car_types,
          edgeFlows: snapshot.edge_flows,
          gameTime: snapshot.game_time,
          playerProgress: snapshot.player_progress,
          playerSpeed: snapshot.player_speed_kmh,
          playerEta: snapshot.player_eta_seconds,
          numCars: snapshot.num_cars,
        }),

      playerArrived: (tokensEarned, routeType) => {
        const state = get();
        const wasCompliant = routeType === "SystemSuggested";
        const newHistory = [...state.complianceHistory, wasCompliant].slice(-20);
        const newStreak = wasCompliant ? state.complianceStreak + 1 : 0;
        const tripTime = (state.gameTime - state.tripStartTime) / 60; // minutes

        set({
          mode: "arrived",
          tokens: state.tokens + tokensEarned,
          totalTrips: state.totalTrips + 1,
          complianceHistory: newHistory,
          complianceStreak: newStreak,
          lastTrip: {
            timeMinutes: tripTime,
            distanceKm:
              state.routeOptions?.[state.selectedRouteIndex ?? 0]
                ?.total_distance_km ?? 0,
            avgSpeedKmh: state.playerSpeed,
            tokensEarned,
            routeType,
            wasCompliant,
          },
        });
      },

      setChallenge: (challenge) => set({ activeChallenge: challenge }),
      completeChallenge: (id) =>
        set((state) => ({
          completedChallenges: [...state.completedChallenges, id],
          activeChallenge: null,
        })),
      resetTrip: () =>
        set({
          mode: "choosing_destination",
          playerDestination: null,
          routeOptions: null,
          selectedRouteIndex: null,
          playerProgress: 0,
          lastTrip: null,
        }),
    }),
    {
      name: "routeshift-game",
      partialize: (state) => ({
        tokens: state.tokens,
        totalTrips: state.totalTrips,
        complianceHistory: state.complianceHistory,
        complianceStreak: state.complianceStreak,
        completedChallenges: state.completedChallenges,
      }),
    }
  )
);
