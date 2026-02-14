import type { ComponentKind } from "../../types/schema";

const KIND_COLORS: Record<ComponentKind, string> = {
  model: "#3b82f6",
  service: "#22c55e",
  transport: "#f97316",
  transform: "#a855f7",
};

interface GroupLabelData {
  label: string;
  count: number;
  kind: ComponentKind;
}

export function GroupLabelNode({ data }: { data: GroupLabelData }) {
  const color = KIND_COLORS[data.kind];
  return (
    <div className="flex items-center gap-2 pointer-events-none select-none">
      <div
        className="w-3 h-3 rounded-sm"
        style={{ backgroundColor: color }}
      />
      <span className="text-sm font-semibold text-gray-300">
        {data.label}
      </span>
      <span className="text-xs text-gray-500">({data.count})</span>
    </div>
  );
}
