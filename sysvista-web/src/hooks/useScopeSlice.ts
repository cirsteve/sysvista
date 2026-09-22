import { useMemo } from "react";
import type { ScopeId, Snapshot } from "../types/v2";
import { indexSnapshot } from "../lib/projection/children";
import { projectScope } from "../lib/projection/project";
import type { ScopeRenderer } from "../lib/livid/types";

export function useScopeSlice(snapshot: Snapshot | null, scopeId: ScopeId, renderer: ScopeRenderer) {
  return useMemo(() => {
    if (!snapshot) return null;
    const index = indexSnapshot(snapshot);
    const projected = projectScope(snapshot, index, scopeId);
    return { index, projected, spec: renderer.toDiagramSpec({ snapshot, index, projected }) };
  }, [snapshot, scopeId, renderer]);
}
