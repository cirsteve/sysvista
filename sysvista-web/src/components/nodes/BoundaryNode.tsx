import { Handle, Position } from "@xyflow/react";
import type { GraphNode } from "../../lib/graph-adapter";

export function BoundaryNode({ data }: { data: GraphNode }) {
  return (
    <div className="rounded-lg border border-dashed border-gray-500 bg-gray-900/90 px-4 py-2 text-gray-300 shadow-lg min-w-[160px]">
      <Handle type="target" position={Position.Top} />
      <div className="text-xs uppercase tracking-wide text-gray-500">External</div>
      <div className="truncate text-sm font-medium">{data.component.name}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
