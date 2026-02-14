import { Handle, Position, type NodeProps } from "@xyflow/react";
import { Database } from "lucide-react";
import type { GraphNode } from "../../lib/graph-adapter";

export function ModelNode({ data }: NodeProps) {
  const { component, hubTier, degree } = data as unknown as GraphNode;
  const isHighHub = hubTier === "high";
  const isMediumHub = hubTier === "medium";

  return (
    <div
      className={`rounded-xl border-2 border-blue-500/50 bg-blue-950/80 px-4 py-2 shadow-lg min-w-[160px] ${
        isHighHub
          ? "shadow-blue-400/40 ring-2 ring-blue-400/60"
          : isMediumHub
            ? "shadow-blue-400/20 ring-1 ring-blue-400/30"
            : "shadow-blue-500/10"
      }`}
    >
      <Handle type="target" position={Position.Top} className="!bg-blue-400" />
      <div className="flex items-center gap-2">
        <Database className="h-4 w-4 text-blue-400 shrink-0" />
        <div className="truncate">
          <div className="text-sm font-semibold text-blue-100 truncate">
            {component.name}
          </div>
          <div className="text-xs text-blue-400/70">{component.language}</div>
        </div>
        {isHighHub && (
          <span className="ml-auto text-[10px] font-bold text-blue-300 bg-blue-800/60 rounded-full px-1.5 py-0.5 shrink-0">
            {degree}
          </span>
        )}
      </div>
      <Handle type="source" position={Position.Bottom} className="!bg-blue-400" />
    </div>
  );
}
