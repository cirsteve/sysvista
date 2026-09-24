import type { CodeEntity, EntityId, ScopeId } from "../../types/v2";
import type { AggregateRelationship, BoundaryNode } from "./types";

export function boundaryNodes(scopeId: ScopeId, children: CodeEntity[], relationships: AggregateRelationship[]): BoundaryNode[] {
  const childIds = new Set<EntityId>(children.map(({ id }) => id));
  const targets = new Set<EntityId>();
  for (const relationship of relationships) {
    if (!childIds.has(relationship.source)) targets.add(relationship.source);
    if (!childIds.has(relationship.target)) targets.add(relationship.target);
  }
  return [...targets].sort().map((target) => ({
    id: `proj:${scopeId}:${target}`, kind: "boundary", name: target, externalTargetId: target,
  }));
}
