"use client";

import { AssignmentResult } from "@/types/graph";

interface StatsPanelProps {
  result: AssignmentResult | null;
  loading: boolean;
  wasmLoading: boolean;
  wasmError: string | null;
  dataError: string | null;
}

export function StatsPanel({
  result,
  loading,
  wasmLoading,
  wasmError,
  dataError,
}: StatsPanelProps) {
  if (wasmError) {
    return (
      <div className="text-red-400 text-xs p-2 bg-red-500/10 rounded-md">
        WASM Error: {wasmError}
      </div>
    );
  }

  if (dataError) {
    return (
      <div className="text-red-400 text-xs p-2 bg-red-500/10 rounded-md">
        Data Error: {dataError}
      </div>
    );
  }

  if (wasmLoading || loading) {
    return (
      <div className="text-white/50 text-xs">
        {wasmLoading ? "Loading WASM..." : "Loading map data..."}
      </div>
    );
  }

  if (!result) {
    return (
      <div className="text-white/40 text-xs">
        Run an assignment to see results
      </div>
    );
  }

  const avgFlow =
    result.edge_flows.reduce((a, b) => a + b, 0) / result.edge_flows.length;
  const maxFlow = Math.max(...result.edge_flows);
  const activeEdges = result.edge_flows.filter((f) => f > 0).length;

  return (
    <div className="grid grid-cols-2 gap-2 text-xs">
      <Stat
        label="System Travel Time"
        value={result.total_system_travel_time.toFixed(0)}
        unit="min"
      />
      <Stat label="Iterations" value={String(result.iterations)} />
      <Stat
        label="Converged"
        value={result.converged ? "Yes" : "No"}
        color={result.converged ? "text-emerald-400" : "text-amber-400"}
      />
      <Stat label="Relative Gap" value={result.relative_gap.toFixed(4)} />
      <Stat label="Avg Flow" value={avgFlow.toFixed(1)} unit="veh" />
      <Stat label="Max Flow" value={maxFlow.toFixed(1)} unit="veh" />
      <Stat label="Active Edges" value={String(activeEdges)} />
      <Stat
        label="Total Edges"
        value={String(result.edge_flows.length)}
      />
    </div>
  );
}

function Stat({
  label,
  value,
  unit,
  color,
}: {
  label: string;
  value: string;
  unit?: string;
  color?: string;
}) {
  return (
    <div className="bg-white/5 rounded-md p-2">
      <div className="text-white/40 text-[10px] uppercase tracking-wide">
        {label}
      </div>
      <div className={`text-sm font-mono ${color ?? "text-white"}`}>
        {value}
        {unit && <span className="text-white/40 ml-1">{unit}</span>}
      </div>
    </div>
  );
}
