import type { Snapshot } from "../../types/v2";
import type { ScopeIndex } from "./types";

export function indexSnapshot(snapshot: Snapshot): ScopeIndex {
  const scopes = new Map<string, { scope_id: never; child_ids: string[]; owner_map: Record<string, string>; crossing_relationship_ids: never[] }>();
  for (const entity of snapshot.entities ?? []) {
    const scope = scopes.get(entity.scope_id) ?? { scope_id: entity.scope_id as never, child_ids: [], owner_map: {}, crossing_relationship_ids: [] };
    scope.child_ids.push(entity.id);
    if (entity.owner_id) scope.owner_map[entity.id] = entity.owner_id;
    scopes.set(entity.scope_id, scope);
  }
  const entityScopes = new Map((snapshot.entities ?? []).map((entity) => [entity.id, entity.scope_id]));
  for (const relationship of snapshot.relationships ?? []) {
    const sourceScope = entityScopes.get(relationship.source);
    const targetScope = entityScopes.get(relationship.target);
    if (sourceScope !== targetScope) {
      for (const id of [sourceScope, targetScope]) if (id && scopes.has(id)) scopes.get(id)!.crossing_relationship_ids.push(relationship.id as never);
    }
  }
  const rootScope = typeof snapshot.root_scope_id === "string" ? snapshot.root_scope_id : "scope:root";
  {
    const root = scopes.get(rootScope) ?? { scope_id: rootScope as never, child_ids: [], owner_map: {}, crossing_relationship_ids: [] };
    root.child_ids = (snapshot.entities ?? []).filter((entity) => entity.declaration_kind !== "file").map(({ id }) => id);
    root.crossing_relationship_ids = (snapshot.relationships ?? []).map(({ id }) => id as never);
    scopes.set(rootScope, root);
  }
  return { scopes: [...scopes.values()].map((scope) => ({ ...scope, child_ids: [...new Set(scope.child_ids)].sort(), crossing_relationship_ids: [...new Set(scope.crossing_relationship_ids)].sort() })).sort((a, b) => String(a.scope_id).localeCompare(String(b.scope_id))) };
}

export function selectChildren(snapshot: Snapshot, index: ScopeIndex, scopeId: string) {
  const slice = index.scopes.find((scope) => scope.scope_id === scopeId);
  if (!slice) return [];
  const ids = new Set(slice.child_ids);
  return (snapshot.entities ?? []).filter((entity) => ids.has(entity.id) || entity.scope_id === scopeId)
    .filter((entity) => !entity.owner_id || ids.has(entity.id))
    .sort((a, b) => a.id.localeCompare(b.id));
}
