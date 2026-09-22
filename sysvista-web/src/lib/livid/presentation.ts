import type { CodeEntity, LogicalModule, SourceFile } from "../../types/v2";
import type { DiagramNode, PresentationType } from "./types";

const entityPresentation = (entity: CodeEntity): PresentationType =>
  entity.declaration_kind === "file" ? "file" : "symbol";

export function entityNode(entity: CodeEntity, childScopeId?: string): DiagramNode {
  const name = String(entity.name);
  return {
    id: entity.id,
    presentation: entityPresentation(entity) as "file" | "symbol",
    label: name,
    ...(childScopeId ? { deferredChildKey: `scope:${childScopeId}` } : {}),
    details: {
      name,
      qualifiedName: String(entity.qualified_name),
      declarationKind: entity.declaration_kind,
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
    details: { path, language: file.language ?? "unknown", analysis: file.analysis },
  };
};
