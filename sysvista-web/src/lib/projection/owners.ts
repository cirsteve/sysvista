import type { CodeEntity, EntityId } from "../../types/v2";

export function mapDescendantsToOwners(
  entities: CodeEntity[],
  children: CodeEntity[],
  indexedOwners: Record<string, EntityId> = {},
): Map<EntityId, EntityId> {
  const childIds = new Set<EntityId>(children.map(({ id }) => id));
  const parents = new Map<EntityId, EntityId | undefined>(entities.map((entity) => [entity.id, indexedOwners[entity.id] ?? entity.owner_id ?? undefined]));
  const result = new Map<EntityId, EntityId>();
  for (const entity of entities) {
    let current: EntityId | undefined = entity.id;
    const seen = new Set<EntityId>();
    while (current && !seen.has(current)) {
      seen.add(current);
      if (childIds.has(current)) { result.set(entity.id, current); break; }
      current = parents.get(current);
    }
  }
  for (const child of children) result.set(child.id, child.id);
  return result;
}
