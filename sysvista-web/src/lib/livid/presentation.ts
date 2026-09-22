import type { CodeEntity, LogicalModule, SourceFile } from "../../types/v2";
import type { DiagramNode } from "./types";

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
