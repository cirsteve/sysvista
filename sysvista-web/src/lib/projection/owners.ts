import type { CodeEntity } from "../../types/v2";

export function mapDescendantsToOwners(
  entities: CodeEntity[],
  children: CodeEntity[],
  indexedOwners: Record<string, string> = {},
): Map<string, string> {
  const childIds = new Set<string>(children.map(({ id }) => id));
  const parents = new Map<string, string | undefined>(entities.map((entity) => [entity.id, indexedOwners[entity.id] ?? entity.owner_id ?? undefined]));
  const result = new Map<string, string>();
  for (const entity of entities) {
    let current: string | undefined = entity.id;
    const seen = new Set<string>();
    while (current && !seen.has(current)) {
      seen.add(current);
      if (childIds.has(current)) { result.set(entity.id, current); break; }
      current = parents.get(current);
    }
  }
  for (const child of children) result.set(child.id, child.id);
  return result;
}
