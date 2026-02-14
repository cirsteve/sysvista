import { Handle, Position, type NodeProps } from "@xyflow/react";
import { Globe } from "lucide-react";
import type { GraphNode } from "../../lib/graph-adapter";

export function TransportNode({ data }: NodeProps) {
  const { component, hubTier, degree } = data as unknown as GraphNode;
  const isHighHub = hubTier === "high";
  const isMediumHub = hubTier === "medium";

  return (
    <div
      className={`border-2 border-orange-500/50 bg-orange-950/80 px-4 py-2 shadow-lg min-w-[160px] ${
        isHighHub
          ? "shadow-orange-400/40 ring-2 ring-orange-400/60"
          : isMediumHub
            ? "shadow-orange-400/20 ring-1 ring-orange-400/30"
            : "shadow-orange-500/10"
      }`}
      style={{ clipPath: "polygon(10% 0%, 90% 0%, 100% 50%, 90% 100%, 10% 100%, 0% 50%)" }}
    >
      <Handle type="target" position={Position.Top} className="!bg-orange-400" />
      <div className="flex items-center gap-2 px-2">
        <Globe className="h-4 w-4 text-orange-400 shrink-0" />
        <div className="truncate">
          <div className="text-sm font-semibold text-orange-100 truncate">
            {component.name}
          </div>
          <div className="text-xs text-orange-400/70">
            {component.transport_protocol ?? component.language}
          </div>
        </div>
        {isHighHub && (
          <span className="ml-auto text-[10px] font-bold text-orange-300 bg-orange-800/60 rounded-full px-1.5 py-0.5 shrink-0">
            {degree}
          </span>
        )}
      </div>
      <Handle type="source" position={Position.Bottom} className="!bg-orange-400" />
    </div>
  );
}
