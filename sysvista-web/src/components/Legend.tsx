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

export function Legend() {
  return (
    <div className="absolute bottom-4 left-14 bg-gray-900/90 backdrop-blur border border-gray-700 rounded-lg px-3 py-2 z-10">
      <div className="flex items-center gap-4">
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
      </div>
    </div>
  );
}
