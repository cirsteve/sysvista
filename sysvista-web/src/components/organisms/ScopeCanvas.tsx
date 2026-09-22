import { useEffect, useState, type ComponentType } from "react";
import type { DiagramSpec, Viewport } from "../../lib/livid/types";

interface LividDiagramProps {
  spec: DiagramSpec;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onDescend: (key: string) => void;
  onViewportChange: (viewport: Viewport) => void;
}

interface ScopeCanvasProps extends LividDiagramProps { notice?: string }

export function ScopeCanvas({ spec, selectedId, onSelect, onDescend, onViewportChange, notice }: ScopeCanvasProps) {
  const [LividDiagram, setLividDiagram] = useState<ComponentType<LividDiagramProps> | null>(null);

  useEffect(() => {
    let active = true;
    const load = async () => {
      try {
        const reactPackage = "@rankonelabs/livid-react";
        const cssPackage = "@rankonelabs/livid-react/styles.css";
        // Keep LividDiagram and its interaction-critical library CSS imports together.
        const [module] = await Promise.all([
          import(/* @vite-ignore */ reactPackage),
          import(/* @vite-ignore */ cssPackage),
        ]);
        const component = (module as Record<string, unknown>).LividDiagram;
        if (active && typeof component === "function") setLividDiagram(() => component as ComponentType<LividDiagramProps>);
      } catch { /* The adapter's visible fallback notice explains why the fake surface is active. */ }
    };
    void load();
    return () => { active = false; };
  }, []);

  if (LividDiagram) return <LividDiagram spec={spec} selectedId={selectedId} onSelect={onSelect} onDescend={onDescend} onViewportChange={onViewportChange} />;

  return (
    <div className="h-full overflow-auto bg-[var(--canvas)] p-8" onScroll={(event) => onViewportChange({ x: event.currentTarget.scrollLeft, y: event.currentTarget.scrollTop, zoom: 1 })}>
      {notice && <div role="status" className="mb-4 rounded border border-amber-300 bg-amber-50 px-3 py-2 text-sm text-amber-900 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-200">{notice}</div>}
      <div className="grid grid-cols-[repeat(auto-fit,minmax(180px,1fr))] gap-6">
        {spec.nodes.map((node) => (
          <button
            key={node.id}
            type="button"
            onClick={() => onSelect(node.id)}
            onDoubleClick={() => node.deferredChildKey && onDescend(node.deferredChildKey)}
            className={`min-h-24 rounded-lg border p-4 text-left shadow-sm transition ${selectedId === node.id ? "border-sky-500 ring-2 ring-sky-200" : "border-[var(--border)] bg-[var(--surface)] hover:border-sky-400"}`}
          >
            <span className="block text-xs uppercase tracking-wide text-[var(--muted)]">{node.presentation}</span>
            <span className="mt-2 block font-medium">{node.label}</span>
            {node.deferredChildKey && <span className="mt-2 block text-xs text-sky-600">Double-click or Enter to drill in</span>}
          </button>
        ))}
      </div>
      {spec.edges.length > 0 && <div className="mt-8 rounded-lg border border-[var(--border)] bg-[var(--surface)] p-3"><h3 className="mb-2 text-xs font-semibold uppercase text-[var(--muted)]">Dependencies</h3>{spec.edges.map((edge) => <button key={edge.id} onClick={() => onSelect(edge.id)} className="mr-2 mb-2 rounded-full border border-[var(--border)] px-3 py-1 text-xs hover:border-sky-400">{edge.source} → {edge.target} · {edge.label} ×{String(edge.details.count)}</button>)}</div>}
    </div>
  );
}
