import { Database, Server, Globe, ArrowRightLeft } from "lucide-react";
import { KIND_COLORS } from "../../lib/design-tokens";
import type { ComponentKind } from "../../types/schema";

const items: { kind: ComponentKind; label: string; Icon: typeof Database }[] = [
  { kind: "model", label: "Model", Icon: Database },
  { kind: "service", label: "Service", Icon: Server },
  { kind: "transport", label: "Transport", Icon: Globe },
  { kind: "transform", label: "Transform", Icon: ArrowRightLeft },
];

const edgeItems = [
  { label: "Payload", color: "bg-pink-400" },
  { label: "Calls", color: "bg-green-400" },
  { label: "Dispatches", color: "bg-amber-400" },
];

const flowEdgeItems = [
  { label: "Payload", color: "bg-pink-400" },
  { label: "Calls", color: "bg-green-400" },
  { label: "Dispatches", color: "bg-amber-400" },
  { label: "Flow", color: "bg-cyan-400" },
];

interface LegendProps {
  mode?: "graph" | "flow";
}

export function Legend({ mode = "graph" }: LegendProps) {
  return (
    <div className="absolute bottom-4 left-14 bg-gray-900/90 backdrop-blur border border-gray-700 rounded-lg px-3 py-2 z-10">
      <div className="flex items-center gap-4">
        {items.map(({ kind, label, Icon }) => (
          <div key={kind} className="flex items-center gap-1.5">
            <Icon className={`h-3.5 w-3.5 ${KIND_COLORS[kind].text}`} />
            <span className="text-xs text-gray-400">{label}</span>
          </div>
        ))}
        {mode === "graph" && (
          <div className="flex items-center gap-1.5">
            <div className="w-3.5 h-3.5 rounded-full ring-2 ring-amber-400/60 bg-amber-950" />
            <span className="text-xs text-gray-400">Hub</span>
          </div>
        )}
        {(mode === "flow" ? flowEdgeItems : edgeItems).map(({ label, color }) => (
          <div key={label} className="flex items-center gap-1.5">
            <div className={`w-5 h-0.5 ${color} rounded`} />
            <span className="text-xs text-gray-400">{label}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
