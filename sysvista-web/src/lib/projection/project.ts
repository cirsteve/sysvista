import type { ScopeId, Snapshot } from "../../types/v2";
import { aggregateCrossingRelationships, summarizeInternalRelationships } from "./aggregate";
import { boundaryNodes } from "./boundary";
import { selectChildren } from "./children";
import { hiddenCounts } from "./hidden";
import { mapDescendantsToOwners } from "./owners";
import type { ProjectedScope, ScopeIndex } from "./types";

export function projectScope(snapshot: Snapshot, index: ScopeIndex, scopeId: ScopeId): ProjectedScope {
  const slice = index.scopes.find((scope) => scope.scope_id === scopeId);
  const children = selectChildren(snapshot, index, scopeId);
  const ownerByEntity = mapDescendantsToOwners(snapshot.entities ?? [], children, slice?.owner_map);
  const scopedIds = new Set(slice?.crossing_relationship_ids ?? []);
  const relationships = slice
    ? (snapshot.relationships ?? []).filter((relationship) =>
        scopedIds.has(relationship.id) ||
        (ownerByEntity.has(relationship.source) && ownerByEntity.has(relationship.target)))
    : [];
  const aggregated = aggregateCrossingRelationships(scopeId, relationships, ownerByEntity);
  const internal = summarizeInternalRelationships(relationships, ownerByEntity);
  const internalIds = relationships
    .filter((relationship) => {
      const sourceOwner = ownerByEntity.get(relationship.source);
      return sourceOwner !== undefined && sourceOwner === ownerByEntity.get(relationship.target);
    })
    .map(({ id }) => id);
  const boundaries = boundaryNodes(scopeId, children, aggregated);
  const visibleIds = new Set([...ownerByEntity.keys(), ...children.map(({ id }) => id)]);
  return { scopeId, children, ownerByEntity, relationships: aggregated,
    internalRelationships: internal,
    boundaryNodes: boundaries, hidden: hiddenCounts(snapshot, visibleIds, aggregated, internalIds) };
}
