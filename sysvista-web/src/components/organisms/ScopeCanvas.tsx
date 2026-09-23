import { useEffect, useRef, useState } from "react";
import { LividDiagram, type LividDiagramHandle } from "@rankonelabs/livid-react";
// Livid's interaction styling must ship beside the component import.
import "@rankonelabs/livid-react/styles.css";
import type { DiagramSpec, RenderedDiagram, Viewport } from "../../lib/livid/types";
import { ChildList } from "./ChildList";
import { resolveFocus, resolveSelection, selectionFrame } from "../../lib/livid/selection";

interface ScopeCanvasProps {
  spec: DiagramSpec;
  diagram: RenderedDiagram | null;
  selectedId: string | null;
  selectedIds?: readonly string[];
  viewport: Viewport;
  onSelect: (id: string | null) => void;
  onDescend: (key: string) => void;
  onViewportChange: (viewport: Viewport) => void;
  notice?: string;
}

export function ScopeCanvas({ spec, diagram, selectedId, selectedIds = [], viewport, onSelect, onDescend, onViewportChange, notice }: ScopeCanvasProps) {
  const livid = useRef<LividDiagramHandle>(null);
  const fallback = useRef<HTMLDivElement>(null);
  const [focusNotice, setFocusNotice] = useState<string>();
  const hasRestoredViewport = viewport.x !== 0 || viewport.y !== 0 || viewport.zoom !== 1;
  useEffect(() => {
    if (diagram && hasRestoredViewport) {
      void livid.current?.setViewport(viewport);
    } else if (!diagram && fallback.current) {
      fallback.current.scrollLeft = viewport.x;
      fallback.current.scrollTop = viewport.y;
    }
  }, [diagram, hasRestoredViewport, viewport]);
  const recordViewport = () => {
    requestAnimationFrame(() => {
      const current = livid.current?.getViewport();
      if (current) onViewportChange(current);
    });
  };

  if (spec.overBudget) return <ChildList items={spec.items ?? []} selectedId={selectedId} onSelect={onSelect} onDescend={onDescend} />;

  if (diagram) {
    return (
      <div className="relative h-full" onPointerUp={recordViewport} onWheel={recordViewport}>
        {(notice || focusNotice) && <div role="status" className="absolute left-4 top-4 z-10 rounded border border-amber-300 bg-amber-50 px-3 py-2 text-sm text-amber-900 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-200">{focusNotice || notice}</div>}
        <LividDiagram
          ref={livid}
          diagram={diagram as never}
          frame={selectionFrame(spec, selectedIds) as never}
          selection={resolveSelection(spec, selectedId) as never}
          focus={resolveFocus(spec, selectedId) as never}
          className="h-full"
          ariaLabel={`Dependency diagram for ${String(spec.scopeId)}`}
          interactive
          fitView={!hasRestoredViewport}
          onSelect={(selection) => onSelect(String(selection.id))}
          onSelectionChange={(selection) => onSelect(selection ? String(selection.id) : null)}
          onFocusResult={(result) => { if (!result.ok) setFocusNotice(`Selection ${String(result.error.focus.id)} is unavailable in this scope`); else setFocusNotice(undefined); }}
          onDescendRequest={(request) => {
            if (request.childState.kind === "deferred") onDescend(String(request.childState.key));
          }}
        />
      </div>
    );
  }

  return (
    <div ref={fallback} className="h-full overflow-auto bg-[var(--canvas)] p-8" onScroll={(event) => onViewportChange({ x: event.currentTarget.scrollLeft, y: event.currentTarget.scrollTop, zoom: 1 })}>
      {notice && <div role="status" className="mb-4 rounded border border-amber-300 bg-amber-50 px-3 py-2 text-sm text-amber-900 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-200">{notice}</div>}
      <div className="grid grid-cols-[repeat(auto-fit,minmax(180px,1fr))] gap-6">
        {spec.nodes.map((node) => (
          <button key={node.id} type="button" aria-pressed={selectedId === node.id || selectedIds.includes(node.id)} onClick={() => onSelect(node.id)} onDoubleClick={() => node.deferredChildKey && onDescend(node.deferredChildKey)} className={`min-h-24 rounded-lg border p-4 text-left shadow-sm transition ${selectedId === node.id || selectedIds.includes(node.id) ? "border-sky-500 ring-2 ring-sky-200" : "border-[var(--border)] bg-[var(--surface)] hover:border-sky-400"}`}>
            <span className="block text-xs uppercase tracking-wide text-[var(--muted)]">{node.presentation}</span>
            <span className="mt-2 block font-medium">{node.label}</span>
            {node.deferredChildKey && <span className="mt-2 block text-xs text-sky-600">Double-click or Enter to drill in</span>}
          </button>
        ))}
      </div>
      {spec.edges.length > 0 && <div className="mt-8 rounded-lg border border-[var(--border)] bg-[var(--surface)] p-3"><h3 className="mb-2 text-xs font-semibold uppercase text-[var(--muted)]">{spec.semanticsProfile === "flow" ? "Direct calls" : "Dependencies"}</h3>{spec.edges.map((edge) => <button key={edge.id} aria-pressed={selectedId === edge.id || (edge.presentation === "aggregate-edge" && edge.details.relationshipIds.includes(selectedId ?? ""))} onClick={() => onSelect(edge.id)} className="mr-2 mb-2 rounded-full border border-[var(--border)] px-3 py-1 text-xs hover:border-sky-400">{edge.source} → {edge.target} · {edge.label}{edge.presentation === "aggregate-edge" ? ` ×${edge.details.count}` : edge.details.backEdge ? " ↩" : ""}</button>)}</div>}
    </div>
  );
}
