import type { ComponentKind } from "../../types/schema";
import { KIND_COLORS } from "../../lib/design-tokens";

interface GroupLabelData {
  label: string;
  count: number;
  kind: ComponentKind;
}

export function GroupLabelNode({ data }: { data: GroupLabelData }) {
  return (
    <div className="flex items-center gap-2 pointer-events-none select-none">
      <div
        className="w-3 h-3 rounded-sm"
        style={{ backgroundColor: KIND_COLORS[data.kind].hex }}
      />
      <span className="text-sm font-semibold text-gray-300">
        {data.label}
      </span>
      <span className="text-xs text-gray-500">({data.count})</span>
    </div>
  );
}
