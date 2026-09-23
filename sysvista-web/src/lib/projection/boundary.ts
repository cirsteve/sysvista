import type { ScopeId } from "../../types/v2";
import type { AggregateRelationship, BoundaryNode } from "./types";
import type { VisibleItem } from "./types";

export function boundaryNodes(scopeId: ScopeId, children: VisibleItem[], relationships: AggregateRelationship[]): BoundaryNode[] {
  const childIds = new Set<string>(children.map(({ id }) => id));
  const targets = new Set<string>();
  for (const relationship of relationships) {
    if (!childIds.has(relationship.source)) targets.add(relationship.source);
    if (!childIds.has(relationship.target)) targets.add(relationship.target);
  }
  return [...targets].sort().map((target) => ({
    id: `proj:${scopeId}:${target}`, kind: "boundary", name: target, externalTargetId: target,
  }));
}
