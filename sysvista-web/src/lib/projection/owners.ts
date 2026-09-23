import type { CodeEntity, EntityId } from "../../types/v2";
import type { VisibleItem } from "./types";

export function mapDescendantsToOwners(
  entities: CodeEntity[],
  children: VisibleItem[],
  indexedOwners: Record<string, string> = {},
): Map<EntityId, string> {
  const visible = new Set(children.map(({ id }) => String(id)));
  const byId = new Map(entities.map((entity) => [entity.id, entity]));
  const result = new Map<EntityId, string>();
  for (const entity of entities) {
    const indexed = indexedOwners[entity.id];
    if (indexed && visible.has(indexed)) { result.set(entity.id, indexed); continue; }
    let current: CodeEntity | undefined = entity;
    const seen = new Set<EntityId>();
    while (current && !seen.has(current.id)) {
      seen.add(current.id);
      if (visible.has(current.id)) { result.set(entity.id, current.id); break; }
      if (current.owner_id) current = byId.get(current.owner_id);
      else {
        if (visible.has(current.file_id)) result.set(entity.id, current.file_id);
        break;
      }
    }
  }
  return result;
}
