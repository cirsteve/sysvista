import type { ComponentKind } from "../../types/schema";

const KINDS: ComponentKind[] = ["model", "service", "transport", "transform", "prompt"];

/** Solid bg classes for active chips (full saturation, not the /20 variants) */
const KIND_SOLID_BG: Record<ComponentKind, string> = {
  model: "bg-blue-500",
  service: "bg-green-500",
  transport: "bg-orange-500",
  transform: "bg-purple-500",
  prompt: "bg-cyan-500",
};

interface FilterChipGroupProps {
  activeKinds: Set<ComponentKind>;
  onToggleKind: (kind: ComponentKind) => void;
}

export function FilterChipGroup({ activeKinds, onToggleKind }: FilterChipGroupProps) {
  return (
    <div className="flex gap-1">
      {KINDS.map((kind) => {
        const active = activeKinds.has(kind);
        return (
          <button
            key={kind}
            onClick={() => onToggleKind(kind)}
            className={`px-2 py-1 text-xs rounded-full border transition-colors ${
              active
                ? `${KIND_SOLID_BG[kind]} text-white border-transparent`
                : "bg-gray-800 text-gray-400 border-gray-700 hover:border-gray-500"
            }`}
          >
            {kind}
          </button>
        );
      })}
    </div>
  );
}
