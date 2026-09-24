import { ChevronLeft, ChevronRight } from "lucide-react";

interface BreadcrumbsProps {
  scopeId: string;
  canBack: boolean;
  canForward: boolean;
  onBack: () => void;
  onForward: () => void;
}

export function Breadcrumbs({ scopeId, canBack, canForward, onBack, onForward }: BreadcrumbsProps) {
  return (
    <nav aria-label="Scope history" className="flex items-center gap-1 text-sm">
      <button className="rounded p-1 disabled:opacity-30" disabled={!canBack} onClick={onBack} aria-label="Back"><ChevronLeft className="h-4 w-4" /></button>
      <button className="rounded p-1 disabled:opacity-30" disabled={!canForward} onClick={onForward} aria-label="Forward"><ChevronRight className="h-4 w-4" /></button>
      <span className="ml-1 font-mono text-xs text-[var(--muted)]">{scopeId}</span>
    </nav>
  );
}
