import type { EntityId, Snapshot } from "../types/v2";
import type { DiagramEdge, DiagramSpec } from "./livid/types";
import type { ViewState } from "./view-state/types";

export const selectVisibleNodes = (spec: DiagramSpec, state: ViewState) => {
  const kinds = new Set(state.filters.kinds);
  return kinds.size === 0 ? spec.nodes : spec.nodes.filter((node) => kinds.has(node.presentation));
};

export const selectCounts = (spec: DiagramSpec, state: ViewState) => ({
  visible: selectVisibleNodes(spec, state).length,
  total: spec.nodes.length,
});

export const selectEvidenceComposition = (edge: DiagramEdge): Record<string, number> => {
  if (!Array.isArray(edge.details.origins)) {
    return { [String(edge.details.origin ?? "unknown")]: Number(edge.details.count ?? 1) };
  }
  const origins = edge.details.origins;
  return origins.reduce<Record<string, number>>((counts, origin) => {
    const key = String(origin ?? "unknown");
    counts[key] = (counts[key] ?? 0) + 1;
    return counts;
  }, {});
};

export const selectEntity = (snapshot: Snapshot, id: string | null) =>
  (snapshot.entities ?? []).find((entity) => entity.id === id) ?? null;

export const owningScopeForEntity = (snapshot: Snapshot, id: EntityId | string) =>
  (snapshot.entities ?? []).find((entity) => entity.id === id)?.scope_id ?? null;
