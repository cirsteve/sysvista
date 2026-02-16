import { Handle, Position, type NodeProps } from "@xyflow/react";
import { ArrowRightLeft } from "lucide-react";
import type { GraphNode } from "../../lib/graph-adapter";

export function TransformNode({ data }: NodeProps) {
  const { component, hubTier, degree, direction } = data as unknown as GraphNode;
  const isHighHub = hubTier === "high";
  const isMediumHub = hubTier === "medium";
  const targetPos = direction === "LR" ? Position.Left : Position.Top;
  const sourcePos = direction === "LR" ? Position.Right : Position.Bottom;

  return (
    <div
      className={`border-2 border-purple-500/50 bg-purple-950/80 px-4 py-3 shadow-lg min-w-[160px] ${
        isHighHub
          ? "shadow-purple-400/40 ring-2 ring-purple-400/60"
          : isMediumHub
            ? "shadow-purple-400/20 ring-1 ring-purple-400/30"
            : "shadow-purple-500/10"
      }`}
      style={{ clipPath: "polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)" }}
    >
      <Handle type="target" position={targetPos} className="!bg-purple-400" />
      <div className="flex items-center justify-center gap-2 px-4">
        <ArrowRightLeft className="h-4 w-4 text-purple-400 shrink-0" />
        <div className="truncate">
          <div className="text-xs font-semibold text-purple-100 truncate">
            {component.name}
          </div>
        </div>
        {isHighHub && (
          <span className="text-[10px] font-bold text-purple-300 bg-purple-800/60 rounded-full px-1.5 py-0.5 shrink-0">
            {degree}
          </span>
        )}
      </div>
      <Handle type="source" position={sourcePos} className="!bg-purple-400" />
    </div>
  );
}
