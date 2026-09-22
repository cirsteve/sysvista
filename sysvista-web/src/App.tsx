import { useState } from "react";
import { Breadcrumbs } from "./components/organisms/Breadcrumbs";
import { Inspector } from "./components/organisms/Inspector";
import { Legend } from "./components/organisms/Legend";
import { ScopeCanvas } from "./components/organisms/ScopeCanvas";
import { ScopeHeader } from "./components/organisms/ScopeHeader";
import { SearchBar } from "./components/organisms/SearchBar";
import { Toolbar } from "./components/organisms/Toolbar";
import { useGraphData } from "./hooks/useGraphData";
import { useKeyboardNavigation } from "./hooks/useKeyboardNavigation";
import { useViewState } from "./hooks/useViewState";
import { useViewStore } from "./store/viewStore";
import type { Manifest } from "./types/v2";

export default function App() {
  useViewState();
  const graph = useGraphData();
  const [error, setError] = useState<string>();
  const history = useViewStore((state) => state.history);
  const back = useViewStore((state) => state.back);
  const forward = useViewStore((state) => state.forward);
  const theme = useViewStore((state) => state.theme);
  const toggleTheme = useViewStore((state) => state.toggleTheme);

  useKeyboardNavigation({ spec: graph.slice?.spec ?? null, selectedId: graph.view.selection, onSelect: graph.select, onDescend: graph.descend, onBack: back });

  const snapshot = graph.loaded?.snapshot;
  const manifest = snapshot?.manifest as Manifest | undefined;
  return (
    <div className={theme === "dark" ? "dark app-shell" : "app-shell"}>
      <Toolbar projectName={manifest ? String(manifest.repository) : undefined} theme={theme} onToggleTheme={toggleTheme} onLoad={graph.load} onError={setError} />
      <div className="flex items-center justify-between border-b border-[var(--border)] bg-[var(--surface)] px-4 py-2">
        <Breadcrumbs scopeId={graph.view.scopeId} canBack={history.cursor > 0} canForward={history.cursor < history.entries.length - 1} onBack={back} onForward={forward} />
        <ScopeHeader title={String(graph.view.scopeId)} visible={graph.counts.visible} total={graph.counts.total} />
        <SearchBar query={graph.view.filters.query} results={graph.results} onSearch={graph.setQuery} onSelect={(hit) => graph.navigateScope(hit.owningScopeId, hit.entityId)} />
      </div>
      <main className="flex min-h-0 flex-1">
        <section className="min-w-0 flex-1">
          {graph.slice ? <ScopeCanvas spec={graph.slice.spec} selectedId={graph.view.selection} onSelect={graph.select} onDescend={graph.descend} onViewportChange={graph.setViewport} notice={graph.rendererNotice} /> : <div className="grid h-full place-items-center text-center text-[var(--muted)]"><div><p className="text-xl font-medium">No architecture loaded</p><p className="mt-1 text-sm">Load a SysVista JSON file or v2 bundle to review it.</p></div></div>}
        </section>
        <Inspector item={graph.selectedItem} />
      </main>
      <Legend />
      {error && <div role="alert" className="fixed bottom-12 left-1/2 -translate-x-1/2 rounded bg-red-700 px-4 py-2 text-sm text-white">{error}</div>}
    </div>
  );
}
