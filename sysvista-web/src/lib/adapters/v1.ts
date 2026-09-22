import type { DetectedComponent, DetectedEdge, SysVistaOutput } from "../../types/schema";
import type { Claim, CodeEntity, EntityId, Evidence, FileId, Relationship, RelationshipId, ScopeId, Snapshot } from "../../types/v2";

const safe = (value: string) => encodeURIComponent(value.replaceAll("\\", "/"));
const fileId = (path: string) => `legacy-file:${safe(path)}` as FileId;
const scopeId = (path: string) => `legacy-scope:${safe(path)}` as ScopeId;
const entityId = (id: string) => `legacy-entity:${safe(id)}` as EntityId;

const edgeKind = (edge: DetectedEdge): Relationship["kind"] => {
  switch (edge.label) {
    case "imports": case "references": case "calls": case "contains": case "depends_on":
    case "handles": case "persists": case "transforms": case "consumes": case "produces":
    case "dispatches": case "invokes_prompt": return edge.label;
    default: return "references";
  }
};

const toEntity = (component: DetectedComponent): CodeEntity => {
  const file = fileId(component.source.file);
  return {
    id: entityId(component.id), name: component.name, qualified_name: component.name,
    declaration_kind: component.kind, file_id: file, scope_id: scopeId(component.source.file),
    owner_id: `legacy-file-entity:${safe(component.source.file)}` as EntityId,
    span: { file_id: file, start_line: component.source.line_start ?? 1, start_column: 1,
      end_line: component.source.line_end ?? component.source.line_start ?? 1, end_column: 1 },
    attributes: { ...component },
  };
};

export function adaptV1(input: SysVistaOutput): Snapshot {
  const components = [...input.components].sort((a, b) => a.id.localeCompare(b.id));
  const edges = [...input.edges].sort((a, b) => `${a.from_id}\0${a.to_id}\0${a.label ?? ""}`.localeCompare(`${b.from_id}\0${b.to_id}\0${b.label ?? ""}`));
  const paths = [...new Set(components.map(({ source }) => source.file))].sort();
  const evidence: Evidence[] = edges.map((edge, index) => ({
    id: `legacy-evidence:${index}`, kind: "analyzer", analyzer: "legacy-v1",
    detail: edge.label === "persists" ? "model_name_match" : "heuristic", origin: "heuristic",
    ...(edge.label === "persists" && { rule: "model_name_match" }),
  }));
  const relationships = edges.map((edge, index) => ({
    id: `legacy-relationship:${index}` as RelationshipId, kind: edgeKind(edge),
    source: entityId(edge.from_id), target: entityId(edge.to_id), origin: "heuristic",
    evidence_id: evidence[index].id,
    ...(edge.label === "persists" && { rule: "model_name_match" }),
  })) as Relationship[];
  const claims: Claim[] = [...(input.workflows ?? [])].sort((a, b) => a.id.localeCompare(b.id)).map((workflow) => {
    const entityIds = [...new Set(workflow.steps.map((step) => step.component_id))].sort().map(entityId);
    const relationshipIds = relationships.filter((relationship) => entityIds.includes(relationship.source) && entityIds.includes(relationship.target)).map(({ id }) => id).sort();
    return { id: `legacy-workflow:${safe(workflow.id)}`, subject: entityId(workflow.entry_point_id),
      predicate: "HeuristicTraversal", object: { name: workflow.name, entity_ids: entityIds, relationship_ids: relationshipIds } };
  });
  const fileEntities: CodeEntity[] = paths.map((path) => ({
    id: `legacy-file-entity:${safe(path)}` as EntityId, name: path.split("/").at(-1) ?? path,
    qualified_name: path, declaration_kind: "file", file_id: fileId(path),
    scope_id: "legacy-scope:root" as ScopeId,
    span: { file_id: fileId(path), start_line: 1, start_column: 1, end_line: 1, end_column: 1 },
    attributes: { synthetic: true, physical_path: path },
  }));
  return {
    manifest: { schema_version: "2", repository: input.project_name, scanned_at: input.scanned_at,
      root: input.root_dir, tool_version: `legacy-v1:${input.version}`,
      analyzer_versions: { legacy: input.version }, inventory: { included: 0, excluded: 0, unsupported: 0, unreadable: 0, failed: 0 },
      legacy: true, coverage: "unknown" },
    coverage: "unknown",
    diagnostics: [{ kind: "warning", message: "Legacy v1 scan: omitted source facts are unrecoverable; coverage is unknown." }],
    source_files: paths.map((path) => ({ id: fileId(path), path, analysis: { kind: "none" }, language: components.find((component) => component.source.file === path)?.language })),
    root_scope_id: "legacy-scope:root",
    entities: [...fileEntities, ...components.map(toEntity)], relationships, evidence, claims,
    modules: [], payload_contracts: [], projections: [], findings: [], unresolved_references: [],
  };
}
