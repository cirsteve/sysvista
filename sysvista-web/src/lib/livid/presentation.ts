import type { CodeEntity, LogicalModule, SourceFile } from "../../types/v2";
import type { FlowGraph } from "../flow/types";
import type { DiagramNode, DiagramSpec } from "./types";
import { FLOW_SEMANTICS_PROFILE } from "./types";

const entityPresentation = (entity: CodeEntity): "file" | "symbol" =>
  entity.declaration_kind === "file" ? "file" : "symbol";

export function entityNode(entity: CodeEntity, childScopeId?: string): DiagramNode {
  const name = String(entity.name);
  if (entityPresentation(entity) === "file") {
    return {
      id: entity.id,
      presentation: "file",
      label: name,
      ...(childScopeId ? { deferredChildKey: `scope:${childScopeId}` } : {}),
      details: { path: String(entity.qualified_name), language: "unknown", analysis: { kind: "none" } },
    };
  }
  return {
    id: entity.id,
    presentation: "symbol",
    label: name,
    ...(childScopeId ? { deferredChildKey: `scope:${childScopeId}` } : {}),
    details: {
      name,
      qualifiedName: String(entity.qualified_name),
      declarationKind: String(entity.declaration_kind),
      fileId: entity.file_id,
      span: entity.span,
    },
  };
}

export const moduleNode = (module: LogicalModule): DiagramNode => ({
  id: module.id,
  presentation: "module",
  label: String(module.name),
  details: { name: String(module.name), fileIds: module.file_ids ?? [], entityIds: module.entity_ids ?? [] },
});

export const fileNode = (file: SourceFile): DiagramNode => {
  const path = String(file.path);
  return {
    id: file.id,
    presentation: "file",
    label: path.split("/").at(-1) ?? path,
    details: { path, language: String(file.language ?? "unknown"), analysis: file.analysis },
  };
};

export function flowPresentation(flow: FlowGraph, scopeId: import("../../types/v2").ScopeId): DiagramSpec {
  return {
    id: `flow:${flow.rootIds.join(",")}:${flow.horizon}`,
    scopeId,
    semanticsProfile: FLOW_SEMANTICS_PROFILE,
    nodes: flow.nodes.map((node) => ({
      id: node.id,
      presentation: "flow" as const,
      label: node.label,
      details: {
        depth: node.depth,
        branch: node.branch,
        truncationCount: node.truncationCount,
        unknownContinuations: node.unknownContinuations,
      },
    })),
    edges: flow.edges.map((edge) => ({
      id: edge.id,
      presentation: "flow-edge" as const,
      source: edge.source,
      target: edge.target,
      label: edge.backEdge ? "cycle" : "calls",
      details: {
        backEdge: edge.backEdge,
        argumentPayloads: edge.argumentPayloads,
        returnPayloads: edge.returnPayloads,
      },
    })),
  };
}
