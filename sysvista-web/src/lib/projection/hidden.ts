import type { EntityId, Snapshot } from "../../types/v2";
import type { AggregateRelationship, HiddenCounts } from "./types";

export function hiddenCounts(snapshot: Snapshot, visibleEntityIds: Set<EntityId>, visibleRelationships: AggregateRelationship[]): HiddenCounts {
  const visibleUnderlying = new Set(visibleRelationships.flatMap(({ relationshipIds }) => relationshipIds));
  return {
    entities: Math.max(0, (snapshot.entities?.length ?? 0) - visibleEntityIds.size),
    relationships: Math.max(0, (snapshot.relationships?.length ?? 0) - visibleUnderlying.size),
  };
}
