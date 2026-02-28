import { Handle, Position, type NodeProps } from "@xyflow/react";
import { MessageSquare } from "lucide-react";
import type { GraphNode } from "../../lib/graph-adapter";

export function PromptNode({ data }: NodeProps) {
  const { component, hubTier, degree, direction } = data as unknown as GraphNode;
  const isHighHub = hubTier === "high";
  const isMediumHub = hubTier === "medium";
  const isFlowMode = direction === "LR";
  const targetPos = isFlowMode ? Position.Left : Position.Top;
  const sourcePos = isFlowMode ? Position.Right : Position.Bottom;

  return (
    <div
      className={`rounded-lg border-2 border-cyan-500/50 bg-cyan-950/80 px-4 py-2 shadow-lg min-w-[180px] ${
        isHighHub
          ? "shadow-cyan-400/40 ring-2 ring-cyan-400/60"
          : isMediumHub
            ? "shadow-cyan-400/20 ring-1 ring-cyan-400/30"
            : "shadow-cyan-500/10"
      } ${isFlowMode ? "prompt-pulse" : ""}`}
    >
      <Handle type="target" position={targetPos} className="!bg-cyan-400" />
      <div className="flex items-center gap-2">
        <MessageSquare className="h-4 w-4 text-cyan-400 shrink-0" />
        <div className="truncate">
          <div className="text-sm font-semibold text-cyan-100 truncate">
            {component.name}
          </div>
          <div className="flex items-center gap-1.5">
            <span className="text-xs text-cyan-400/70">{component.language}</span>
            {component.prompt_subtype && (
              <span className="text-[10px] font-medium text-cyan-300 bg-cyan-800/60 rounded-full px-1.5 py-0.5">
                {component.prompt_subtype}
              </span>
            )}
          </div>
        </div>
        {isHighHub && (
          <span className="ml-auto text-[10px] font-bold text-cyan-300 bg-cyan-800/60 rounded-full px-1.5 py-0.5 shrink-0">
            {degree}
          </span>
        )}
      </div>
      <Handle type="source" position={sourcePos} className="!bg-cyan-400" />
    </div>
  );
}
