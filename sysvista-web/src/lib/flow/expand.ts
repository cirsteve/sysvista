import type { CodeEntity, EntityId, Relationship, Snapshot } from "../../types/v2";
import { annotatePayloads } from "./annotate";
import type { FlowEdge, FlowGraph, FlowNode, FlowRoot, UnknownContinuation } from "./types";

interface QueueItem { id: EntityId; depth: number; path: EntityId[] }

const relationshipOrigin = (relationship: Relationship): string => String(relationship.origin ?? "").toLowerCase();
const isPartial = (relationship: Relationship) => relationshipOrigin(relationship).includes("partial");
const isHeuristic = (relationship: Relationship) => relationshipOrigin(relationship).includes("heuristic");

function rootsFor(snapshot: Snapshot, root: FlowRoot): EntityId[] {
  if (root.kind === "entity") return [root.entityId];
  const contract = (snapshot.payload_contracts ?? []).find(({ name }) => name === root.contractName);
  return [...new Set([...(contract?.producer_ids ?? []), ...(contract?.consumer_ids ?? [])])] as EntityId[];
}

export function expandFlow(snapshot: Snapshot, root: FlowRoot, horizon = 3): FlowGraph {
  const boundedHorizon = Math.max(0, Math.floor(horizon));
  const entities = new Map<EntityId, CodeEntity>((snapshot.entities ?? []).map((entity) => [entity.id, entity]));
  const calls = (snapshot.relationships ?? []).filter((relationship) => relationship.kind === "calls");
  const outgoing = calls.reduce((map, relationship) => {
    const list = map.get(relationship.source) ?? [];
    list.push(relationship);
    map.set(relationship.source, list);
    return map;
  }, new Map<EntityId, Relationship[]>());
  const unresolved = (snapshot.unresolved_references ?? []).reduce((map, item) => {
    const list = map.get(item.source) ?? [];
    list.push(item);
    map.set(item.source, list);
    return map;
  }, new Map<EntityId, NonNullable<Snapshot["unresolved_references"]>>());
  const rootIds = rootsFor(snapshot, root).filter((id) => entities.has(id));
  const queue: QueueItem[] = rootIds.map((id) => ({ id, depth: 0, path: [id] }));
  const depths = new Map<EntityId, number>(rootIds.map((id) => [id, 0]));
  const flowEdges = new Map<string, FlowEdge>();
  const unknownByNode = new Map<EntityId, UnknownContinuation[]>();
  const truncation = new Map<EntityId, number>();

  while (queue.length > 0) {
    const current = queue.shift()!;
    const relationships = outgoing.get(current.id) ?? [];
    const unknowns: UnknownContinuation[] = [
      ...(unknownByNode.get(current.id) ?? []),
      ...(unresolved.get(current.id) ?? []).map((item, index) => ({ id: `unresolved:${current.id}:${index}`, name: String(item.name), reason: "unresolved" as const })),
    ];
    const known = relationships.filter((relationship) => {
      if (isHeuristic(relationship)) {
        unknowns.push({ id: relationship.id, name: String(entities.get(relationship.target)?.name ?? relationship.target), reason: "heuristic" });
        return false;
      }
      if (isPartial(relationship) || !entities.has(relationship.target)) {
        unknowns.push({ id: relationship.id, name: String(entities.get(relationship.target)?.name ?? relationship.target), reason: "partial" });
        return false;
      }
      return true;
    });
    unknownByNode.set(current.id, unknowns);
    known.forEach((relationship) => {
      const backEdge = current.path.includes(relationship.target);
      if (current.depth >= boundedHorizon && !backEdge) {
        truncation.set(current.id, (truncation.get(current.id) ?? 0) + 1);
        return;
      }
      const payloads = annotatePayloads(snapshot, relationship.source, relationship.target);
      flowEdges.set(relationship.id, { id: relationship.id, source: relationship.source, target: relationship.target, backEdge, ...payloads });
      if (backEdge) return;
      const nextDepth = current.depth + 1;
      if (!depths.has(relationship.target) || nextDepth < (depths.get(relationship.target) ?? Infinity)) {
        depths.set(relationship.target, nextDepth);
        queue.push({ id: relationship.target, depth: nextDepth, path: [...current.path, relationship.target] });
      }
    });
  }

  const outgoingVisible = [...flowEdges.values()].reduce((map, edge) => map.set(edge.source, (map.get(edge.source) ?? 0) + (edge.backEdge ? 0 : 1)), new Map<EntityId, number>());
  const nodes = [...depths].map<FlowNode>(([id, depth]) => ({
    id,
    label: String(entities.get(id)?.name ?? id),
    depth,
    branch: (outgoingVisible.get(id) ?? 0) > 1,
    truncationCount: truncation.get(id) ?? 0,
    unknownContinuations: unknownByNode.get(id) ?? [],
  })).sort((a, b) => a.depth - b.depth || a.id.localeCompare(b.id));
  return { horizon: boundedHorizon, rootIds, nodes, edges: [...flowEdges.values()].sort((a, b) => a.id.localeCompare(b.id)) };
}
