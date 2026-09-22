import type { EntityId, ScopeId, Snapshot } from "../../types/v2";
import type { ScopeIndex, ScopeSlice } from "./types";

const toScopeId = (value: string): ScopeId => value as ScopeId;
const VIEWER_ROOT_SCOPE = toScopeId("scope:root");

const emptyScope = (scopeId: ScopeId): ScopeSlice => ({
  scope_id: scopeId,
  child_ids: [],
  owner_map: {},
  crossing_relationship_ids: [],
});

const scopeFor = (scopes: Map<ScopeId, ScopeSlice>, scopeId: ScopeId): ScopeSlice => {
  const existing = scopes.get(scopeId);
  if (existing) return existing;
  const created = emptyScope(scopeId);
  scopes.set(scopeId, created);
  return created;
};

export function rootScopeId(snapshot: Snapshot): ScopeId {
  return typeof snapshot.root_scope_id === "string"
    ? toScopeId(snapshot.root_scope_id)
    : VIEWER_ROOT_SCOPE;
}

export function indexSnapshot(snapshot: Snapshot): ScopeIndex {
  const scopes = new Map<ScopeId, ScopeSlice>();
  for (const entity of snapshot.entities ?? []) {
    const scope = scopeFor(scopes, entity.scope_id);
    scope.child_ids.push(entity.id);
    if (entity.owner_id) scope.owner_map[entity.id] = entity.owner_id;
  }

  const entityScopes = new Map<EntityId, ScopeId>(
    (snapshot.entities ?? []).map((entity) => [entity.id, entity.scope_id]),
  );
  for (const relationship of snapshot.relationships ?? []) {
    const sourceScope = entityScopes.get(relationship.source);
    const targetScope = entityScopes.get(relationship.target);
    if (sourceScope === targetScope) continue;
    for (const scopeId of [sourceScope, targetScope]) {
      if (!scopeId) continue;
      const scope = scopes.get(scopeId);
      if (scope) scope.crossing_relationship_ids.push(relationship.id);
    }
  }

  const root = scopeFor(scopes, rootScopeId(snapshot));
  root.child_ids = (snapshot.entities ?? [])
    .filter((entity) => entity.declaration_kind !== "file")
    .map(({ id }) => id);
  root.crossing_relationship_ids = (snapshot.relationships ?? []).map(({ id }) => id);

  return {
    scopes: [...scopes.values()]
      .map((scope) => ({
        ...scope,
        child_ids: [...new Set(scope.child_ids)].sort(),
        crossing_relationship_ids: [...new Set(scope.crossing_relationship_ids)].sort(),
      }))
      .sort((a, b) => a.scope_id.localeCompare(b.scope_id)),
  };
}

export function selectChildren(snapshot: Snapshot, index: ScopeIndex, scopeId: ScopeId) {
  const slice = index.scopes.find((scope) => scope.scope_id === scopeId);
  if (!slice) return [];
  const ids = new Set<string>(slice.child_ids);
  return (snapshot.entities ?? [])
    .filter((entity) => ids.has(entity.id) || entity.scope_id === scopeId)
    .filter((entity) => !entity.owner_id || ids.has(entity.id))
    .sort((a, b) => a.id.localeCompare(b.id));
}
