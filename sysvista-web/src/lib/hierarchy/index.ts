import type { EntityId, ScopeId, Snapshot } from "../../types/v2";
import type { ScopeChild, ScopeIndex, ScopeSlice } from "../../types/v2";

export interface HierarchyIndex {
  readonly rootScopeId: ScopeId;
  readonly scopes: ReadonlyMap<ScopeId, ScopeSlice>;
  readonly parents: ReadonlyMap<ScopeId, ScopeId>;
  readonly items: ReadonlyMap<string, ScopeChild>;
  readonly entityScopes: ReadonlyMap<EntityId, ScopeId>;
  readonly memberships: ReadonlyMap<string, ReadonlySet<ScopeId>>;
  scopeOf(id: string): ScopeId | undefined;
  parentOf(id: ScopeId): ScopeId | undefined;
  itemsOf(id: ScopeId): readonly ScopeChild[];
  representativeOf(scopeId: ScopeId, entityId: EntityId): string | undefined;
  nearestValidScope(id: ScopeId): ScopeId;
}

const cache = new WeakMap<Snapshot, HierarchyIndex>();

export function buildHierarchyIndex(snapshot: Snapshot): HierarchyIndex {
  const existing = cache.get(snapshot);
  if (existing) return existing;
  const rootScopeId = snapshot.manifest.root_scope_id as ScopeId;
  const source = snapshot.scope_index as ScopeIndex | undefined;
  const scopes = new Map<ScopeId, ScopeSlice>((source?.scopes ?? []).map((slice) => [slice.scope_id as ScopeId, slice]));
  const parents = new Map<ScopeId, ScopeId>();
  for (const projection of snapshot.projections ?? []) {
    if (projection.parent_scope_id) parents.set(projection.scope_id, projection.parent_scope_id as ScopeId);
  }
  const items = new Map<string, ScopeChild>();
  const memberships = new Map<string, Set<ScopeId>>();
  for (const slice of scopes.values()) {
    for (const child of slice.children) {
      const id = child.kind === "file" ? child.file_id : child.kind === "module" ? child.module_id
        : child.kind === "symbol" ? child.entity_id : child.scope_id;
      items.set(id, child);
      const set = memberships.get(id) ?? new Set<ScopeId>();
      set.add(slice.scope_id as ScopeId);
      memberships.set(id, set);
      if (child.scope_id && !parents.has(child.scope_id as ScopeId))
        parents.set(child.scope_id as ScopeId, slice.scope_id as ScopeId);
    }
  }
  const entityScopes = new Map((snapshot.entities ?? []).map((entity) => [entity.id, entity.scope_id]));
  const scopeOf = (id: string) => entityScopes.get(id as EntityId) ?? [...(memberships.get(id) ?? [])][0];
  const nearestValidScope = (id: ScopeId) => {
    let current: ScopeId | undefined = id;
    const seen = new Set<ScopeId>();
    while (current && !seen.has(current)) {
      if (scopes.has(current)) return current;
      seen.add(current);
      current = parents.get(current);
    }
    return rootScopeId;
  };
  const index: HierarchyIndex = {
    rootScopeId, scopes, parents, items, entityScopes, memberships,
    scopeOf,
    parentOf: (id) => parents.get(id),
    itemsOf: (id) => scopes.get(id)?.children ?? [],
    representativeOf: (scopeId, entityId) => scopes.get(scopeId)?.owner_map?.[entityId],
    nearestValidScope,
  };
  cache.set(snapshot, index);
  return index;
}
