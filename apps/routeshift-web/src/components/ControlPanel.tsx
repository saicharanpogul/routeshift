"use client";

import { AssignmentType } from "@/types/graph";

interface ControlPanelProps {
  assignmentType: AssignmentType;
  onAssignmentTypeChange: (type: AssignmentType) => void;
  onRun: () => void;
  running: boolean;
  wasmReady: boolean;
  dataReady: boolean;
}

export function ControlPanel({
  assignmentType,
  onAssignmentTypeChange,
  onRun,
  running,
  wasmReady,
  dataReady,
}: ControlPanelProps) {
  const canRun = wasmReady && dataReady && !running;

  return (
    <div className="flex flex-col gap-3">
      <div>
        <label className="text-xs text-white/60 uppercase tracking-wide">
          Assignment Mode
        </label>
        <div className="flex gap-1 mt-1">
          <button
            onClick={() => onAssignmentTypeChange("UserEquilibrium")}
            className={`flex-1 px-2 py-1.5 text-xs rounded-md transition-colors ${
              assignmentType === "UserEquilibrium"
                ? "bg-blue-500 text-white"
                : "bg-white/10 text-white/70 hover:bg-white/20"
            }`}
          >
            Selfish (UE)
          </button>
          <button
            onClick={() => onAssignmentTypeChange("SystemOptimal")}
            className={`flex-1 px-2 py-1.5 text-xs rounded-md transition-colors ${
              assignmentType === "SystemOptimal"
                ? "bg-emerald-500 text-white"
                : "bg-white/10 text-white/70 hover:bg-white/20"
            }`}
          >
            Optimal (SO)
          </button>
        </div>
      </div>

      <button
        onClick={onRun}
        disabled={!canRun}
        className={`w-full py-2 rounded-md text-sm font-medium transition-colors ${
          canRun
            ? "bg-white text-black hover:bg-white/90"
            : "bg-white/10 text-white/30 cursor-not-allowed"
        }`}
      >
        {running ? "Computing..." : "Run Assignment"}
      </button>
    </div>
  );
}
