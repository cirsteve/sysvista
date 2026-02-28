import { ChevronRight } from "lucide-react";
import type { ComponentKind } from "../../types/schema";
import { KindDot } from "../atoms/KindDot";
import type { ReactNode } from "react";

interface ListItemProps {
  kind?: ComponentKind;
  label: string;
  sublabel?: string;
  onClick: () => void;
  showChevron?: boolean;
  /** Optional leading content (e.g. a step badge) instead of KindDot */
  leading?: ReactNode;
}

export function ListItem({ kind, label, sublabel, onClick, showChevron, leading }: ListItemProps) {
  return (
    <button
      onClick={onClick}
      className="w-full text-left px-2 py-1.5 rounded text-sm hover:bg-gray-800 transition-colors flex items-center gap-2"
    >
      {leading ?? (kind && <KindDot kind={kind} />)}
      <div className="min-w-0 flex-1">
        <div className="text-sm text-gray-300 truncate">{label}</div>
        {sublabel && (
          <div className="text-xs text-gray-500 truncate">{sublabel}</div>
        )}
      </div>
      {showChevron && (
        <ChevronRight className="h-3.5 w-3.5 text-gray-600 shrink-0" />
      )}
    </button>
  );
}
