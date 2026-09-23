import type { EntityId, Relationship, ScopeId } from "../../types/v2";
import type { AggregateRelationship, InternalRelationshipSummary } from "./types";

export function aggregateCrossingRelationships(
  scopeId: ScopeId,
  relationships: Relationship[],
  ownerByEntity: Map<EntityId, string>,
): AggregateRelationship[] {
  const groups = new Map<string, AggregateRelationship>();
  for (const relationship of relationships) {
    const source = ownerByEntity.get(relationship.source) ?? relationship.source;
    const target = ownerByEntity.get(relationship.target) ?? relationship.target;
    if (source === target) continue;
    const key = `${source}\0${target}\0${relationship.kind}\0${relationship.origin}`;
    const existing = groups.get(key);
    if (existing) existing.relationshipIds.push(relationship.id);
    else groups.set(key, { id: `aggregate:${JSON.stringify([scopeId, source, target, relationship.kind, relationship.origin])}`, source, target,
      kind: relationship.kind, origin: String(relationship.origin), relationshipIds: [relationship.id] });
  }
  return [...groups.values()].map((item) => ({ ...item, relationshipIds: item.relationshipIds.sort() }))
    .sort((a, b) => a.id.localeCompare(b.id));
}

export function summarizeInternalRelationships(
  relationships: Relationship[], ownerByEntity: Map<EntityId, string>,
): InternalRelationshipSummary[] {
  const summaries = new Map<string, InternalRelationshipSummary>();
  for (const relationship of relationships) {
    const source = ownerByEntity.get(relationship.source);
    const target = ownerByEntity.get(relationship.target);
    if (!source || source !== target) continue;
    const summary = summaries.get(source) ?? { ownerId: source, count: 0, byKind: {} };
    summary.count += 1;
    const kind = String(relationship.kind);
    summary.byKind[kind] = (summary.byKind[kind] ?? 0) + 1;
    summaries.set(source, summary);
  }
  return [...summaries.values()].sort((a, b) => a.ownerId.localeCompare(b.ownerId));
}
