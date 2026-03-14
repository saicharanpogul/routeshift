# RouteShift

**Incentive-Based Traffic Redistribution System**

RouteShift is an open-source traffic redistribution system that uses real-time congestion data and incentive mechanisms to distribute vehicles across optimal routes at a **system level**, not just an individual level.

Current navigation apps (Google Maps, Waze) solve for the individual — they show each user the fastest route. This creates a well-documented problem in traffic engineering called the **Price of Anarchy**, where individually optimal choices produce collectively suboptimal outcomes. When thousands of users get the same "fastest route" suggestion, that route becomes congested, and the system as a whole moves slower.

RouteShift solves this by treating the road network as a coordinated system. It knows how many drivers are on each route, calculates the system-optimal distribution, and incentivizes drivers to choose routes that benefit everyone — including themselves — through token rewards, gamification, and real-time feedback.

> **Honest Disclaimer:** This system cannot be deployed at scale by a solo builder. This repository is the complete blueprint and the simulation is the proof of concept. Fork it, fund it, build it.

---

## What This Project Is

- A **complete open-source specification**: architecture, algorithms, protocols, data models
- A **working simulation** that demonstrates the concept with hundreds of virtual drivers on real maps of Hyderabad, Mumbai, and Bangalore
- A **playable demo** where users can see and interact with the traffic redistribution in action
- An **honest blueprint** for anyone — city, company, or community — who wants to build the real thing

## What This Project Is NOT

- A production-ready app you can deploy tomorrow
- A startup pitch disguised as open source
- A solved problem — the hard parts (adoption, verification, city partnerships) remain open

---

## The Problem

Every major navigation app optimizes for the individual user. This leads to three failures:

1. **Herd routing** — When a previously fast route gets recommended to thousands of users simultaneously, it becomes the new bottleneck
2. **No coordination mechanism** — There is no way for the system to say "we need 200 fewer cars on MG Road in the next 15 minutes" and actually make that happen through user cooperation
3. **Zero incentive alignment** — Users have no reason to take a route that is 3 minutes slower for them but 20 minutes faster for the system aggregate. Altruism does not scale

### The Price of Anarchy

In game theory, the Price of Anarchy (PoA) measures how much worse a system performs when agents act selfishly versus when a central coordinator assigns optimal strategies:

- **Selfish routing (current state):** Each driver picks their shortest path. PoA can be as high as 4/3 for linear latency functions (Roughgarden & Tardos, 2002)
- **System-optimal routing (RouteShift target):** A coordinator assigns routes such that total travel time across all drivers is minimized. Research shows this can reduce total travel time by **10-30%** in congested networks

### Why Indian Cities

| Factor | Impact | Opportunity |
|--------|--------|-------------|
| Extreme density | Small routing changes affect thousands of vehicles | High leverage per user converted |
| Mixed traffic (auto, bike, car, bus) | Navigation apps optimize only for cars | Multi-modal routing is an open space |
| Rapid smartphone adoption | 80%+ urban drivers use navigation | Distribution channel exists |
| Government digitization push | Smart city initiatives actively seeking solutions | Potential institutional buyers |

---

## Architecture

RouteShift is a modular monorepo with clear boundaries between components:

```
routeshift/
├── packages/
│   ├── routeshift-core/        Rust: graph, Dijkstra, Frank-Wolfe, BPR functions
│   ├── routeshift-wasm/        WebAssembly bindings (wasm-bindgen)
│   ├── routeshift-sim/         Agent-based simulation engine (Phase 2)
│   └── routeshift-solana/      Solana reward program (Phase 4)
├── apps/
│   └── routeshift-web/         Next.js frontend with MapLibre GL
└── tools/
    └── routeshift-data/        OSM data extraction (Python/osmnx)
```

### System Layers

| Layer | Component | Purpose | Tech |
|-------|-----------|---------|------|
| Data Ingestion | Traffic Data Aggregator | Ingest real-time traffic from multiple sources | Python, Redis |
| Data Ingestion | Map/Road Graph Engine | Maintain weighted road network graph | OSM data, Rust |
| Intelligence | Congestion Predictor | Forecast congestion 15-60 min ahead | LSTM/GNN models |
| Intelligence | System-Optimal Router | Calculate globally optimal route assignments | Frank-Wolfe algorithm |
| Intelligence | Incentive Engine | Determine reward amounts per route choice | Game theory, dynamic pricing |
| Coordination | Route Assignment Service | Match drivers to routes in real-time | Rust/WASM |
| Interface | Web App | Map visualization, game UI, dashboard | Next.js, MapLibre GL |
| Blockchain | Reward Settlement | Issue and manage incentive tokens | Solana (Anchor) |

### Data Flow

```
OSM (Overpass API)
  → Python/osmnx extraction
  → GeoJSON (map rendering) + Graph JSON (WASM computation)

Browser:
  fetch(graph.json) → serde_wasm_bindgen → Rust Graph struct
    → Frank-Wolfe solver → AssignmentResult
    → Color-code roads by congestion → MapLibre re-renders
```

The system operates in a continuous loop:

1. **INGEST** — Traffic data feeds into the aggregator
2. **MODEL** — Road network graph gets updated edge weights based on congestion
3. **OPTIMIZE** — Frank-Wolfe algorithm computes ideal driver distribution
4. **ASSIGN** — Drivers get route suggestions with incentives attached
5. **VERIFY** — GPS traces confirm route compliance
6. **REWARD** — Solana program issues tokens for verified compliance
7. **FEEDBACK** — Metrics feed back into the model

---

## Core Algorithm

### Frank-Wolfe Traffic Assignment

The mathematical foundation is the System-Optimal Traffic Assignment problem:

**Given** a directed graph G = (V, E) where each edge e has a latency function l_e(x) depending on flow x:

- **Minimize:** Total system travel time = Σ x_e · l_e(x_e) across all edges
- **Subject to:** Flow conservation, non-negativity, demand satisfaction

**BPR Latency Function** (Bureau of Public Roads):

```
l_e(x) = t_e × (1 + 0.15 × (x / c_e)^4)
```

Where `t_e` is free-flow travel time and `c_e` is road capacity.

**Algorithm Steps:**

1. Initialize with all-or-nothing assignment using free-flow times
2. Compute travel times using BPR function
3. Find shortest paths using Dijkstra (the descent direction)
4. Line search for optimal step size
5. Update flows — repeat until convergence (relative gap < 0.01)

The implementation supports both **User Equilibrium** (selfish routing) and **System Optimal** (socially optimal routing), allowing direct comparison of the Price of Anarchy.

### Incentive Engine

Reward for a route choice R:

```
reward(R) = base_reward × congestion_delta(R) × compliance_history(driver) × urgency_multiplier
```

- **congestion_delta** — How much this choice reduces system congestion (1.0x to 5.0x)
- **compliance_history** — Loyalty multiplier based on past behavior (0.8x to 2.0x)
- **urgency_multiplier** — Spike during peak congestion (1.0x to 3.0x)

---

## Tech Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Core Algorithms | Rust | Performance-critical graph algorithms, compiles to WASM |
| WASM Bindings | wasm-bindgen + serde-wasm-bindgen | Zero-copy JS ↔ Rust serialization |
| Frontend | Next.js 16 + TypeScript + Tailwind CSS | Modern React with server components |
| Map Rendering | MapLibre GL JS (OpenFreeMap tiles) | Open-source, no API key needed |
| State Management | Zustand | Lightweight, fast for real-time simulation state |
| Monorepo | Turborepo + pnpm workspaces | Multi-language build orchestration |
| Data Extraction | Python + osmnx | One-line API for OSM road network download |
| Blockchain | Solana (Anchor) | Sub-second finality, negligible tx costs for micro-rewards |

---

## Getting Started

### Prerequisites

- **Rust** (1.88+) with `wasm32-unknown-unknown` target
- **wasm-pack** (`cargo install wasm-pack`)
- **Node.js** (20+) and **pnpm** (9+)
- **Python 3.10+** with osmnx (optional, for real OSM data extraction)

### Setup

```bash
# Clone the repo
git clone https://github.com/saicharanpogul/routeshift.git
cd routeshift

# Install JS dependencies
pnpm install

# Build WASM package
wasm-pack build packages/routeshift-wasm --target web --out-dir pkg

# Run tests
cargo test

# Start dev server
pnpm --filter routeshift-web dev
```

Open [http://localhost:3000](http://localhost:3000) to see the app.

### Extracting Real OSM Data (Optional)

The repo ships with synthetic sample data. To use real road networks from OpenStreetMap:

```bash
cd tools/routeshift-data
pip install -r requirements.txt
python scripts/extract_network.py --city hyderabad  # or mumbai, bangalore, all
```

### Build Pipeline

Turborepo enforces this build order:

```
routeshift-core (Rust) → routeshift-wasm (wasm-pack) → routeshift-web (Next.js)
```

```bash
# Build everything
pnpm build

# Or build individually
cargo build -p routeshift-core --release
wasm-pack build packages/routeshift-wasm --target web --out-dir pkg
pnpm --filter routeshift-web build
```

---

## Simulation

The simulation wraps the core algorithms in an interactive experience:

### Modes

1. **Baseline (Selfish)** — All drivers use User Equilibrium routing. Shows natural congestion
2. **RouteShift (Optimal)** — System-optimal routing with incentives. Side-by-side comparison against baseline

### Controls

- **City selector** — Switch between Hyderabad, Mumbai, Bangalore
- **Assignment mode** — Toggle between Selfish (UE) and Optimal (SO)
- **Run Assignment** — Execute Frank-Wolfe solver via WASM

### Metrics

| Metric | Description |
|--------|-------------|
| System Travel Time | Total travel time across all drivers (lower is better) |
| Iterations | Frank-Wolfe iterations to convergence |
| Relative Gap | Convergence measure (< 0.01 = converged) |
| Active Edges | Road segments carrying traffic |

### Target Performance

| Metric | Baseline (Selfish) | Target (RouteShift) |
|--------|-------------------|---------------------|
| Avg commute time | Measured per city | 15-25% reduction |
| Throughput | Measured per city | 20-30% improvement |
| Road utilization variance | High | 50%+ reduction |

---

## Roadmap

### Phase 1: Foundation (Current)

- [x] Monorepo scaffolding (Turborepo + pnpm + Cargo workspace)
- [x] Rust core: graph data structure, Dijkstra, BPR functions
- [x] Rust core: Frank-Wolfe traffic assignment (UE + SO)
- [x] WASM bindings with wasm-bindgen
- [x] Next.js frontend with MapLibre GL
- [x] Sample road network data for 3 cities
- [ ] Real OSM data extraction via osmnx
- [ ] Vehicle animation on map

### Phase 2: Simulation Engine

- [ ] Agent-based driver model with OD pairs and departure times
- [ ] Time-stepped simulation loop (30-second ticks)
- [ ] Split-screen comparison (selfish vs optimal)
- [ ] Congestion heat map overlay

### Phase 3: Game Layer

- [ ] Commissioner role with budget management
- [ ] Incentive allocation interface
- [ ] Scoring system and leaderboard
- [ ] Incident injection (road closures, accidents)
- [ ] Time controls (pause, play, fast-forward)

### Phase 4: Solana Integration

- [ ] ROUTE token program on devnet
- [ ] Mock wallet integration
- [ ] Compliance verification demonstration
- [ ] Reward settlement via on-chain transactions

---

## Contributing

Explicitly designed for community contribution:

- **Add new cities** — OSM data is available worldwide. Extract the graph and calibrate parameters
- **Improve routing algorithms** — Better algorithms (origin-based, bush-based) would improve convergence
- **ML congestion prediction** — The simulation generates training data for predictive models
- **Mobile app prototype** — React Native prototype to make it tangible
- **Alternative incentive models** — Mechanism design researchers can propose better incentive structures
- **Solana program** — Implement the ROUTE token and reward settlement

### What Cannot Be Built Solo

| Challenge | What Would Be Needed |
|-----------|---------------------|
| Real-world GPS verification at scale | 2-3 backend engineers + GIS specialist, 6+ months |
| City government partnerships | BD/partnerships lead, legal, 6-12 month cycle |
| Critical mass of users | Marketing budget, growth team, local operations |
| Anti-gaming at scale | Security engineer, adversarial testing, bug bounty |
| Multi-modal routing | Transit data partnerships, per-city customization |

---

## Key References

- Roughgarden & Tardos (2002) — *How Bad is Selfish Routing?* — Price of Anarchy foundations
- Wardrop (1952) — *Some Theoretical Aspects of Road Traffic Research* — User Equilibrium vs System Optimal
- Frank & Wolfe (1956) — *An algorithm for quadratic programming* — The optimization algorithm
- Sheffi (1985) — *Urban Transportation Networks* — Traffic assignment textbook
- Newson & Krumm (2009) — *Hidden Markov Map Matching* — GPS trace to road matching

---

## License

MIT License. No restrictions on commercial use. If a city or company wants to build on this, they should.
