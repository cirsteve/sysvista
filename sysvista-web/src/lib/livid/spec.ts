import type { ScopeProjection, DiagramSpec } from "./types";
import { DEPENDENCY_SEMANTICS_PROFILE } from "./types";
import { entityNode, fileNode, moduleNode } from "./presentation";

export function toDiagramSpec({ snapshot, projected }: ScopeProjection): DiagramSpec {
  const visible = new Set(projected.children.map(({ id }) => String(id)));
  const childScopes = new Map<string, string>();
  (snapshot.entities ?? []).forEach((entity) => {
    if (entity.owner_id && visible.has(entity.owner_id) && entity.scope_id !== projected.scopeId) {
      childScopes.set(entity.owner_id, entity.scope_id);
    }
  });
  const entityNodes = projected.children.map((entity) => entityNode(entity, childScopes.get(entity.id)));
  const moduleNodes = (snapshot.modules ?? []).filter(({ id }) => visible.has(id)).map(moduleNode);
  const fileNodes = (snapshot.source_files ?? []).filter(({ id }) => visible.has(id)).map(fileNode);
  const boundaryTarget = new Map(projected.boundaryNodes.map((node) => [String(node.externalTargetId), node.id]));
  const boundaryNodes = projected.boundaryNodes.map((node) => ({
    id: node.id,
    presentation: "boundary" as const,
    label: node.name,
    details: { externalTargetId: node.externalTargetId },
  }));

  return {
    id: `scope:${projected.scopeId}`,
    scopeId: projected.scopeId,
    semanticsProfile: DEPENDENCY_SEMANTICS_PROFILE,
    nodes: [...moduleNodes, ...fileNodes, ...entityNodes, ...boundaryNodes].sort((a, b) => a.id.localeCompare(b.id)),
    edges: projected.relationships.map((edge) => ({
      id: edge.id,
      presentation: "aggregate-edge" as const,
      source: boundaryTarget.get(String(edge.source)) ?? edge.source,
      target: boundaryTarget.get(String(edge.target)) ?? edge.target,
      label: String(edge.kind),
      details: {
        kind: edge.kind,
        origin: edge.origin,
        count: edge.relationshipIds.length,
        relationshipIds: edge.relationshipIds,
      },
    })).sort((a, b) => a.id.localeCompare(b.id)),
  };
}
