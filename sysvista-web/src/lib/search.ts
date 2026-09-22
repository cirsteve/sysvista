import type { EntityId, ScopeId, Snapshot } from "../types/v2";

export interface EntitySearchHit {
  entityId: EntityId;
  name: string;
  qualifiedName: string;
  owningScopeId: ScopeId;
  kind: string;
}

export function searchSnapshot(snapshot: Snapshot, query: string, limit = 20): EntitySearchHit[] {
  const normalized = query.trim().toLocaleLowerCase();
  if (!normalized) return [];
  return (snapshot.entities ?? [])
    .filter((entity) => [entity.name, entity.qualified_name, entity.declaration_kind]
      .some((value) => String(value).toLocaleLowerCase().includes(normalized)))
    .slice(0, limit)
    .map((entity) => ({
      entityId: entity.id,
      name: String(entity.name),
      qualifiedName: String(entity.qualified_name),
      owningScopeId: entity.scope_id,
      kind: String(entity.declaration_kind),
    }));
}
