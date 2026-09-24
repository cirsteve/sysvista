import type { CodeEntity, RelationshipId, Snapshot } from "../../types/v2";
import type { AggregateRelationship, HiddenCounts } from "./types";

export function isDefaultHidden(entity: CodeEntity): boolean { return entity.is_local === true; }

export function hiddenCounts(
  snapshot: Snapshot,
  visibleEntityIds: Set<string>,
  visibleRelationships: AggregateRelationship[],
  summarizedRelationshipIds: RelationshipId[] = [],
): HiddenCounts {
  const visibleUnderlying = new Set([
    ...visibleRelationships.flatMap(({ relationshipIds }) => relationshipIds),
    ...summarizedRelationshipIds,
  ]);
  return {
    entities: Math.max(0, (snapshot.entities?.length ?? 0) - visibleEntityIds.size),
    relationships: Math.max(0, (snapshot.relationships?.length ?? 0) - visibleUnderlying.size),
  };
}
