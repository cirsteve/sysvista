import type { EntityId, ScopeId, Snapshot } from "../types/v2";
import { buildHierarchyIndex } from "./hierarchy/index";

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
  const hierarchy = buildHierarchyIndex(snapshot);
  const pathByFile = new Map((snapshot.source_files ?? []).map((file) => [file.id, file.path.toLocaleLowerCase()]));
  const rank = (name: string, qualified: string, path: string) => {
    const n = name.toLocaleLowerCase();
    const q = qualified.toLocaleLowerCase();
    if (n === normalized || q === normalized) return 0;
    if (n.startsWith(normalized) || q.startsWith(normalized)) return 1;
    if (n.includes(normalized) || q.includes(normalized)) return 2;
    if (path.includes(normalized)) return 3;
    return 4;
  };
  return (snapshot.entities ?? [])
    .filter((entity) => rank(entity.name, entity.qualified_name, pathByFile.get(entity.file_id) ?? "") < 4)
    .sort((a, b) => rank(a.name, a.qualified_name, pathByFile.get(a.file_id) ?? "") - rank(b.name, b.qualified_name, pathByFile.get(b.file_id) ?? "") || a.name.localeCompare(b.name) || a.id.localeCompare(b.id))
    .slice(0, limit)
    .map((entity) => ({
      entityId: entity.id,
      name: String(entity.name),
      qualifiedName: String(entity.qualified_name),
      owningScopeId: hierarchy.scopeOf(entity.id) ?? entity.scope_id,
      kind: String(entity.declaration_kind),
    }));
}
