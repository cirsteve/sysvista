import type { ScopeProjection, DiagramSpec, DiagramNode } from "./types";
import { DEPENDENCY_SEMANTICS_PROFILE } from "./types";
import { entityNode, fileNode, moduleNode } from "./presentation";

export function toDiagramSpec({ snapshot, projected }: ScopeProjection): DiagramSpec {
  if (projected.children.length + projected.boundaryNodes.length > 300) {
    return { id: `scope:${projected.scopeId}`, scopeId: projected.scopeId,
      semanticsProfile: DEPENDENCY_SEMANTICS_PROFILE, nodes: [], edges: [],
      overBudget: true, items: projected.children };
  }
  const files = new Map((snapshot.source_files ?? []).map((file) => [file.id, file]));
  const modules = new Map((snapshot.modules ?? []).map((module) => [module.id, module]));
  const nodes: DiagramNode[] = projected.children.flatMap((item): DiagramNode[] => {
    switch (item.kind) {
      case "directory":
        return [{ id: item.id, presentation: "directory", label: item.name, deferredChildKey: `scope:${item.scopeId}`, details: { name: item.name } }];
      case "file": {
        const file = files.get(item.id);
        const node = file && fileNode(file);
        return node ? [{ ...node, deferredChildKey: `scope:${item.scopeId}` }] : [];
      }
      case "module": {
        const module = modules.get(item.id);
        const node = module && moduleNode(module);
        return node ? [{ ...node, deferredChildKey: `scope:${item.scopeId}` }] : [];
      }
      case "symbol": return item.entity ? [entityNode(item.entity, item.scopeId)] : [];
    }
  });
  const boundaryTarget = new Map(projected.boundaryNodes.map((node) => [String(node.externalTargetId), node.id]));
  const boundaryNodes: DiagramNode[] = projected.boundaryNodes.map((node) => ({
    id: node.id,
    presentation: "boundary",
    label: node.name,
    details: { externalTargetId: node.externalTargetId },
  }));
  return {
    id: `scope:${projected.scopeId}`,
    scopeId: projected.scopeId,
    semanticsProfile: DEPENDENCY_SEMANTICS_PROFILE,
    nodes: [...nodes, ...boundaryNodes].sort((a, b) => a.id.localeCompare(b.id)),
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
