import type { ScopeId, Snapshot } from "../../types/v2";
import type { ScopeIndex, ScopeSlice, VisibleItem } from "./types";
import { buildHierarchyIndex } from "../hierarchy/index";
import { isDefaultHidden } from "./hidden";

const toScopeId = (value: string): ScopeId => value as ScopeId;
const projectionCache = new WeakMap<Snapshot, ScopeIndex>();

export function rootScopeId(snapshot: Snapshot): ScopeId {
  return toScopeId(snapshot.manifest.root_scope_id ?? (typeof snapshot.root_scope_id === "string" ? snapshot.root_scope_id : "scope:root"));
}

export function indexSnapshot(snapshot: Snapshot): ScopeIndex {
  const cached = projectionCache.get(snapshot);
  if (cached) return cached;
  const bundled = snapshot.scope_index as ScopeIndex | undefined;
  if (bundled?.scopes) {
    const hierarchy = buildHierarchyIndex(snapshot);
    const index = { scopes: [...hierarchy.scopes.values()].map((scope) => ({ ...scope, child_ids: [] })) } as unknown as ScopeIndex;
    projectionCache.set(snapshot, index);
    return index;
  }
  const root = rootScopeId(snapshot);
  const index = { scopes: [{ scope_id: root, child_scope_ids: [], child_ids: [], children: (snapshot.entities ?? [])
    .map((entity) => ({ kind: "symbol" as const, entity_id: entity.id })),
    owner_map: {}, crossing_relationship_ids: (snapshot.relationships ?? []).map(({ id }) => id) }] };
  projectionCache.set(snapshot, index);
  return index;
}

export function selectChildren(snapshot: Snapshot, index: ScopeIndex, scopeId: ScopeId): VisibleItem[] {
  const slice = index.scopes.find((scope) => scope.scope_id === scopeId) as ScopeSlice | undefined;
  if (!slice) return [];
  const projections = new Map((snapshot.projections ?? []).map((projection) => [projection.scope_id, projection]));
  const files = new Map((snapshot.source_files ?? []).map((file) => [file.id, file]));
  const entities = new Map((snapshot.entities ?? []).map((entity) => [entity.id, entity]));
  const modules = new Map((snapshot.modules ?? []).map((module) => [module.id, module]));
  const raw = slice.children ?? (slice.child_ids ?? []).map((id) => ({ kind: "symbol" as const, entity_id: id, scope_id: null }));
  return raw.flatMap((child): VisibleItem[] => {
    switch (child.kind) {
      case "directory": return [{ kind: "directory", id: toScopeId(child.scope_id), scopeId: toScopeId(child.scope_id), name: projections.get(toScopeId(child.scope_id))?.name ?? child.scope_id }];
      case "file": {
        const file = files.get(child.file_id as import("../../types/v2").FileId);
        return file ? [{ kind: "file", id: file.id, scopeId: toScopeId(child.scope_id), name: file.path }] : [];
      }
      case "module": {
        const module = modules.get(child.module_id as import("../../types/v2").ModuleId);
        return module ? [{ kind: "module", id: module.id, scopeId: toScopeId(child.scope_id), name: module.name }] : [];
      }
      case "symbol": {
        const entity = entities.get(child.entity_id as import("../../types/v2").EntityId);
        return entity && !isDefaultHidden(entity) ? [{ kind: "symbol", id: entity.id, ...(child.scope_id ? { scopeId: toScopeId(child.scope_id) } : {}), name: entity.name, entity }] : [];
      }
    }
  }).sort((a, b) => a.id.localeCompare(b.id));
}
