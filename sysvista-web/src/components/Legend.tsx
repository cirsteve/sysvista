import { Database, Server, Globe, ArrowRightLeft } from "lucide-react";

const items = [
  { kind: "model", label: "Model", color: "text-blue-400", Icon: Database },
  { kind: "service", label: "Service", color: "text-green-400", Icon: Server },
  { kind: "transport", label: "Transport", color: "text-orange-400", Icon: Globe },
  {
    kind: "transform",
    label: "Transform",
    color: "text-purple-400",
    Icon: ArrowRightLeft,
  },
];

const edgeItems = [
  { label: "Payload", color: "bg-pink-400" },
  { label: "Calls", color: "bg-green-400" },
  { label: "Dispatches", color: "bg-amber-400" },
];

interface LegendProps {
  mode?: "graph" | "workflow";
}

const stepItems = [
  { label: "Entry", color: "bg-orange-500" },
  { label: "Call", color: "bg-green-500" },
  { label: "Persist", color: "bg-blue-500" },
  { label: "Dispatch", color: "bg-amber-500" },
  { label: "Response", color: "bg-cyan-500" },
];

export function Legend({ mode = "graph" }: LegendProps) {
  return (
    <div className="absolute bottom-4 left-14 bg-gray-900/90 backdrop-blur border border-gray-700 rounded-lg px-3 py-2 z-10">
      <div className="flex items-center gap-4">
        {mode === "workflow" ? (
          stepItems.map(({ label, color }) => (
            <div key={label} className="flex items-center gap-1.5">
              <div className={`w-3 h-3 rounded-full ${color}`} />
              <span className="text-xs text-gray-400">{label}</span>
            </div>
          ))
        ) : (
          <>
            {items.map(({ kind, label, color, Icon }) => (
              <div key={kind} className="flex items-center gap-1.5">
                <Icon className={`h-3.5 w-3.5 ${color}`} />
                <span className="text-xs text-gray-400">{label}</span>
              </div>
            ))}
            <div className="flex items-center gap-1.5">
              <div className="w-3.5 h-3.5 rounded-full ring-2 ring-amber-400/60 bg-amber-950" />
              <span className="text-xs text-gray-400">Hub</span>
            </div>
            {edgeItems.map(({ label, color }) => (
              <div key={label} className="flex items-center gap-1.5">
                <div className={`w-5 h-0.5 ${color} rounded`} />
                <span className="text-xs text-gray-400">{label}</span>
              </div>
            ))}
          </>
        )}
      </div>
    </div>
  );
}
