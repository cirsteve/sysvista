import { useEffect, useMemo, useRef, useState } from "react";
import type { Manifest, ScopeId, Snapshot } from "../types/v2";
import { indexSnapshot } from "../lib/projection/children";
import { projectScope } from "../lib/projection/project";
import { ScopeRequestCoordinator } from "../lib/livid/requests";
import type { ScopeRenderer, ScopeRenderResult } from "../lib/livid/types";

interface ScopeSliceResult extends ScopeRenderResult {
  index: ReturnType<typeof indexSnapshot>;
  projected: ReturnType<typeof projectScope>;
}

export function useScopeSlice(snapshot: Snapshot | null, scopeId: ScopeId, renderer: ScopeRenderer) {
  const coordinator = useRef(new ScopeRequestCoordinator<ScopeSliceResult>());
  const requestNumber = useRef(0);
  const projection = useMemo(() => {
    if (!snapshot) return null;
    const index = indexSnapshot(snapshot);
    return { index, projected: projectScope(snapshot, index, scopeId) };
  }, [snapshot, scopeId]);
  const baseSlice = useMemo<ScopeSliceResult | null>(() => {
    if (!snapshot || !projection) return null;
    return {
      ...projection,
      spec: renderer.toDiagramSpec({ snapshot, ...projection }),
      diagram: null,
    };
  }, [projection, renderer, snapshot]);
  const [completed, setCompleted] = useState<{
    projection: typeof projection;
    renderer: ScopeRenderer;
    slice: ScopeSliceResult;
  } | null>(null);

  useEffect(() => {
    if (!snapshot || !projection) return;
    const manifest = snapshot.manifest as Manifest;
    const snapshotId = String(manifest.scanned_at ?? manifest.repository);
    const key = { snapshotId, scopeId, requestId: `render-${++requestNumber.current}` };
    const requestCoordinator = coordinator.current;
    requestCoordinator.begin(key);
    void renderer.render({ snapshot, ...projection }).then((result) => {
      requestCoordinator.commit(key, { ...projection, ...result }, (slice) => {
        setCompleted({ projection, renderer, slice });
      });
    });
    return () => requestCoordinator.clear(key);
  }, [projection, renderer, scopeId, snapshot]);

  return completed?.projection === projection && completed.renderer === renderer
    ? completed.slice
    : baseSlice;
}
