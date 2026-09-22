import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { LoadedSnapshot } from "../lib/loader";
import type { Manifest, ScopeId } from "../types/v2";
import type { Viewport } from "../lib/livid/types";
import { loadScopeRenderer } from "../lib/livid/adapter";
import { FakeScopeRenderer } from "../lib/livid/fake";
import type { ScopeRenderer } from "../lib/livid/types";
import { indexSnapshot, rootScopeId } from "../lib/projection/children";
import { searchSnapshot } from "../lib/search";
import { selectCounts } from "../lib/selectors";
import { nearestValidScope } from "../lib/view-state/fallback";
import { useViewStore } from "../store/viewStore";
import { useScopeSlice } from "./useScopeSlice";
import { expandFlow } from "../lib/flow/expand";
import { flowPresentation } from "../lib/livid/presentation";
import type { ScopeRenderResult } from "../lib/livid/types";

export function useGraphData() {
  const [loaded, setLoaded] = useState<LoadedSnapshot | null>(null);
  const [renderer, setRenderer] = useState<ScopeRenderer>(() => new FakeScopeRenderer());
  const [rendererNotice, setRendererNotice] = useState<string>();
  const reportedDiagnostics = useRef(new Set<string>());
  const view = useViewStore((state) => state.view);
  const replaceView = useViewStore((state) => state.replaceView);
  const navigateScope = useViewStore((state) => state.navigateScope);
  const updateView = useViewStore((state) => state.updateView);
  const addDiagnostic = useViewStore((state) => state.addDiagnostic);
  const structuralSlice = useScopeSlice(loaded?.snapshot ?? null, view.scopeId, renderer);
  const flowSpec = useMemo(() => {
    if (!loaded || view.lens !== "flow") return null;
    const selected = (loaded.snapshot.entities ?? []).find(({ id }) => id === view.selection);
    const root = selected ?? structuralSlice?.projected.children.find(({ declaration_kind }) => declaration_kind === "function") ?? structuralSlice?.projected.children[0];
    if (!root) return null;
    return flowPresentation(expandFlow(loaded.snapshot, { kind: "entity", entityId: root.id }, view.flowHops), view.scopeId);
  }, [loaded, structuralSlice?.projected, view.flowHops, view.lens, view.scopeId, view.selection]);
  const [renderedFlow, setRenderedFlow] = useState<{ spec: typeof flowSpec; result: ScopeRenderResult } | null>(null);
  useEffect(() => {
    if (!flowSpec) return;
    let active = true;
    void renderer.renderSpec(flowSpec).then((result) => active && setRenderedFlow({ spec: flowSpec, result }));
    return () => { active = false; };
  }, [flowSpec, renderer]);
  const flowSlice = flowSpec ? (renderedFlow?.spec === flowSpec ? renderedFlow.result : { spec: flowSpec, diagram: null }) : null;
  const slice = view.lens === "flow" ? flowSlice : structuralSlice;

  useEffect(() => {
    void loadScopeRenderer().then((result) => {
      setRenderer(result.renderer);
      if (result.diagnostic) {
        setRendererNotice(result.diagnostic.message);
        addDiagnostic(result.diagnostic);
      }
    });
  }, [addDiagnostic]);

  useEffect(() => {
    const current = slice?.diagnostic;
    if (current && !reportedDiagnostics.current.has(current.message)) {
      reportedDiagnostics.current.add(current.message);
      addDiagnostic(current);
    }
  }, [addDiagnostic, slice?.diagnostic]);

  const load = useCallback((data: LoadedSnapshot) => {
    setLoaded(data);
    const scopeId = rootScopeId(data.snapshot);
    const manifest = data.snapshot.manifest as Manifest;
    const snapshotId = String(manifest.scanned_at ?? manifest.repository);
    const restored = view.snapshotId === snapshotId;
    const index = indexSnapshot(data.snapshot);
    const valid = new Set(index.scopes.map((scope) => scope.scope_id));
    valid.add(scopeId);
    const entities = new Map((data.snapshot.entities ?? []).map((entity) => [entity.id, entity]));
    const parents = new Map(index.scopes.flatMap((scope) => {
      const owner = (data.snapshot.entities ?? []).find((entity) => entity.scope_id === scope.scope_id && entity.owner_id)?.owner_id;
      const parent = owner ? entities.get(owner)?.scope_id : undefined;
      return parent ? [[scope.scope_id, parent] as const] : [];
    }));
    const fallback = nearestValidScope(restored ? view.scopeId : scopeId, valid, parents, scopeId);
    if (fallback.diagnostic) addDiagnostic(fallback.diagnostic);
    replaceView(restored ? { ...view, scopeId: fallback.scopeId } : {
      snapshotId, scopeId: fallback.scopeId,
      filters: { kinds: [], origins: [], query: "" }, selection: null,
      selectedEntities: [], lens: "structure", flowHops: 3,
      viewport: { x: 0, y: 0, zoom: 1 },
    });
  }, [addDiagnostic, replaceView, view]);

  const descend = useCallback((key: string) => navigateScope(key.replace(/^scope:/, "") as ScopeId), [navigateScope]);
  const select = useCallback((id: string | null) => updateView({ selection: id }, false), [updateView]);
  const setViewport = useCallback((viewport: Viewport) => updateView({ viewport }, false), [updateView]);
  const setQuery = useCallback((query: string) => updateView({ filters: { ...view.filters, query } }, false), [updateView, view.filters]);
  const setLens = useCallback((lens: "structure" | "flow") => updateView({ lens }), [updateView]);
  const setFlowHops = useCallback((flowHops: number) => updateView({ flowHops: Math.max(0, Math.min(8, Math.floor(flowHops))) }), [updateView]);
  const navigateFinding = useCallback((target: { scopeId: ScopeId; entityIds: import("../types/v2").EntityId[] }) => {
    replaceView({ ...view, scopeId: target.scopeId, selection: target.entityIds[0] ?? null, selectedEntities: target.entityIds });
  }, [replaceView, view]);
  const results = useMemo(() => loaded ? searchSnapshot(loaded.snapshot, view.filters.query) : [], [loaded, view.filters.query]);
  const selectedItem = useMemo(() => slice?.spec.nodes.find(({ id }) => id === view.selection) ?? slice?.spec.edges.find(({ id }) => id === view.selection) ?? null, [slice, view.selection]);
  const counts = useMemo(() => slice ? selectCounts(slice.spec, view) : { visible: 0, total: 0 }, [slice, view]);

  return { loaded, rendererNotice: slice?.diagnostic?.message ?? rendererNotice, view, slice, results, selectedItem, counts, load, descend, select, setViewport, setQuery, setLens, setFlowHops, navigateScope, navigateFinding };
}
