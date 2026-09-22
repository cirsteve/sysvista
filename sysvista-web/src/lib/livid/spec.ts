import type { ScopeProjection, DiagramSpec } from "./types";
import { DEPENDENCY_SEMANTICS_PROFILE } from "./types";
import { entityNode, fileNode, moduleNode } from "./presentation";

const bundledChildIds = (scopeIndex: unknown, scopeId: string): string[] => {
  if (typeof scopeIndex !== "object" || scopeIndex === null || !("scopes" in scopeIndex)) return [];
  const scopes = (scopeIndex as { scopes?: unknown }).scopes;
  if (!Array.isArray(scopes)) return [];
  const scope = scopes.find((candidate) =>
    typeof candidate === "object" && candidate !== null && "scope_id" in candidate && candidate.scope_id === scopeId,
  );
  if (typeof scope !== "object" || scope === null || !("child_ids" in scope) || !Array.isArray(scope.child_ids)) return [];
  return scope.child_ids.filter((id: unknown): id is string => typeof id === "string");
};

export function toDiagramSpec({ snapshot, index, projected }: ScopeProjection): DiagramSpec {
  const visibleEntities = new Set(projected.children.map(({ id }) => String(id)));
  const indexedChildren = new Set(
    [
      ...(index.scopes.find(({ scope_id }) => scope_id === projected.scopeId)?.child_ids.map(String) ?? []),
      ...bundledChildIds(snapshot.scope_index, projected.scopeId),
    ],
  );
  const modules = (snapshot.modules ?? []).filter((module) =>
    module.scope_id === projected.scopeId ||
    (module.entity_ids ?? []).some((id) => visibleEntities.has(id)) ||
    (module.file_ids ?? []).some((id) => indexedChildren.has(id)),
  );
  const visibleFiles = new Set([
    ...projected.children.map(({ file_id }) => String(file_id)),
    ...modules.flatMap(({ file_ids }) => (file_ids ?? []).map(String)),
    ...indexedChildren,
  ]);
  const childScopes = new Map<string, string>();
  (snapshot.entities ?? []).forEach((entity) => {
    if (entity.owner_id && visibleEntities.has(entity.owner_id) && entity.scope_id !== projected.scopeId) {
      childScopes.set(entity.owner_id, entity.scope_id);
    }
  });
  const entityNodes = projected.children.map((entity) => entityNode(entity, childScopes.get(entity.id)));
  const moduleNodes = modules.map(moduleNode);
  const fileNodes = (snapshot.source_files ?? []).filter(({ id }) => visibleFiles.has(id)).map(fileNode);
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
        kind: String(edge.kind),
        origin: edge.origin,
        count: edge.relationshipIds.length,
        relationshipIds: edge.relationshipIds,
      },
    })).sort((a, b) => a.id.localeCompare(b.id)),
  };
}
