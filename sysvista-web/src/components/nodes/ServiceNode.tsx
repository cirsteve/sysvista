import { Handle, Position, type NodeProps } from "@xyflow/react";
import { Server } from "lucide-react";
import type { GraphNode } from "../../lib/graph-adapter";

export function ServiceNode({ data }: NodeProps) {
  const { component, hubTier, degree } = data as unknown as GraphNode;
  const isHighHub = hubTier === "high";
  const isMediumHub = hubTier === "medium";

  return (
    <div
      className={`rounded-lg border-2 border-green-500/50 bg-green-950/80 px-4 py-2 shadow-lg min-w-[160px] ${
        isHighHub
          ? "shadow-green-400/40 ring-2 ring-green-400/60"
          : isMediumHub
            ? "shadow-green-400/20 ring-1 ring-green-400/30"
            : "shadow-green-500/10"
      }`}
    >
      <Handle type="target" position={Position.Top} className="!bg-green-400" />
      <div className="flex items-center gap-2">
        <Server className="h-4 w-4 text-green-400 shrink-0" />
        <div className="truncate">
          <div className="text-sm font-semibold text-green-100 truncate">
            {component.name}
          </div>
          <div className="text-xs text-green-400/70">{component.language}</div>
        </div>
        {isHighHub && (
          <span className="ml-auto text-[10px] font-bold text-green-300 bg-green-800/60 rounded-full px-1.5 py-0.5 shrink-0">
            {degree}
          </span>
        )}
      </div>
      <Handle type="source" position={Position.Bottom} className="!bg-green-400" />
    </div>
  );
}
