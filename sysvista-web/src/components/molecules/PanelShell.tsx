import { X } from "lucide-react";
import type { ReactNode } from "react";

interface PanelShellProps {
  side: "left" | "right";
  title: string;
  onClose: () => void;
  children: ReactNode;
  headerExtra?: ReactNode;
}

export function PanelShell({ side, title, onClose, children, headerExtra }: PanelShellProps) {
  const positionClass = side === "left"
    ? "left-0 border-r"
    : "right-0 border-l";

  return (
    <div className={`absolute ${positionClass} top-0 h-full w-80 bg-gray-900/95 backdrop-blur border-gray-700 shadow-2xl overflow-y-auto z-50`}>
      <div className="p-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-bold text-gray-100">{title}</h2>
          {headerExtra}
          <button
            onClick={onClose}
            aria-label="Close panel"
            className="p-1 rounded hover:bg-gray-700 text-gray-400 hover:text-gray-200"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}
