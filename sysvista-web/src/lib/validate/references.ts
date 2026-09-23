import type { Claim, Evidence, Finding, LogicalModule, PayloadContract, Projection, Snapshot } from "../../types/v2";
import type { Result } from "../result";
import { buildHierarchyIndex } from "../hierarchy/index";

export interface ReferenceError {
  ownerId: string;
  field: string;
  targetId: string;
  message: string;
}

const missing = (errors: ReferenceError[], ownerId: string, field: string, targetId: string) =>
  errors.push({ ownerId, field, targetId, message: `${ownerId} has dangling ${field} reference to ${targetId}` });

export function validateReferences(snapshot: Snapshot): Result<Snapshot, ReferenceError[]> {
  const errors: ReferenceError[] = [];
  const hierarchy = snapshot.scope_index ? buildHierarchyIndex(snapshot) : undefined;
  const files = new Set<string>((snapshot.source_files ?? []).map(({ id }) => id));
  const entities = new Set<string>((snapshot.entities ?? []).map(({ id }) => id));
  const modules = new Set<string>((snapshot.modules ?? []).map(({ id }) => id));
  if (snapshot.scope_index) {
    if (!hierarchy) throw new Error("hierarchy index unavailable");
    if (!hierarchy.scopes.has(hierarchy.rootScopeId)) missing(errors, "manifest", "root_scope_id", hierarchy.rootScopeId);
    for (const scope of hierarchy.scopes.values()) {
      const visible = new Set(scope.children.map((child) => child.kind === "file" ? child.file_id : child.kind === "module" ? child.module_id : child.kind === "symbol" ? child.entity_id : child.scope_id));
      for (const child of scope.children) {
        if (child.scope_id && !hierarchy.scopes.has(child.scope_id as import("../../types/v2").ScopeId))
          missing(errors, scope.scope_id, "children.scope_id", child.scope_id);
        if (child.kind === "file" && !files.has(child.file_id)) missing(errors, scope.scope_id, "children.file_id", child.file_id);
        if (child.kind === "module" && !modules.has(child.module_id)) missing(errors, scope.scope_id, "children.module_id", child.module_id);
        if (child.kind === "symbol" && !entities.has(child.entity_id)) missing(errors, scope.scope_id, "children.entity_id", child.entity_id);
      }
      for (const [entityId, target] of Object.entries(scope.owner_map)) {
        if (!entities.has(entityId)) missing(errors, scope.scope_id, "owner_map.entity_id", entityId);
        if (!visible.has(target)) missing(errors, entityId, "owner_map", target);
      }
    }
    const active = new Set<string>();
    const done = new Set<string>();
    const walk = (id: string): void => {
      if (active.has(id)) { missing(errors, id, "containment_cycle", id); return; }
      if (done.has(id)) return;
      active.add(id);
      for (const child of hierarchy.itemsOf(id as import("../../types/v2").ScopeId)) {
        if (child.scope_id) walk(child.scope_id);
      }
      active.delete(id); done.add(id);
    };
    walk(hierarchy.rootScopeId);
    for (const scopeId of hierarchy.scopes.keys()) walk(scopeId);
  }
  const relationships = new Set<string>((snapshot.relationships ?? []).map(({ id }) => id));
  const evidenceItems = (snapshot.evidence ?? []) as Evidence[];
  const evidence = new Set(evidenceItems.map(({ id }) => id));
  for (const entity of snapshot.entities ?? []) {
    if (!files.has(entity.file_id)) missing(errors, entity.id, "file_id", entity.file_id);
    if (entity.owner_id && !entities.has(entity.owner_id)) missing(errors, entity.id, "owner_id", entity.owner_id);
    if (!files.has(entity.span.file_id)) missing(errors, entity.id, "span.file_id", entity.span.file_id);
  }
  for (const relationship of snapshot.relationships ?? []) {
    if (!entities.has(relationship.source)) missing(errors, relationship.id, "source", relationship.source);
    if (!entities.has(relationship.target)) missing(errors, relationship.id, "target", relationship.target);
    if (relationship.evidence_id && !evidence.has(String(relationship.evidence_id))) missing(errors, relationship.id, "evidence_id", String(relationship.evidence_id));
  }
  for (const claim of (snapshot.claims ?? []) as Claim[]) {
    if (!entities.has(claim.subject)) missing(errors, String(claim.id), "subject", claim.subject);
    for (const id of (claim.evidence_ids ?? []) as string[]) if (!evidence.has(id)) missing(errors, String(claim.id), "evidence_ids", id);
    const object = claim.object as unknown as Record<string, unknown>;
    const entityIds = Array.isArray(object.entity_ids) ? object.entity_ids : [];
    const relationshipIds = Array.isArray(object.relationship_ids) ? object.relationship_ids : [];
    for (const id of entityIds.map(String)) if (!entities.has(id)) missing(errors, String(claim.id), "object.entity_ids", id);
    for (const id of relationshipIds.map(String)) if (!relationships.has(id)) missing(errors, String(claim.id), "object.relationship_ids", id);
  }
  for (const item of snapshot.unresolved_references ?? []) {
    if (!entities.has(item.source)) missing(errors, `unresolved:${item.name}`, "source", item.source);
    if (!files.has(item.span.file_id)) missing(errors, `unresolved:${item.name}`, "span.file_id", item.span.file_id);
  }
  for (const projection of (snapshot.projections ?? []) as Projection[]) {
    for (const id of (projection.entity_ids ?? []) as string[]) if (!entities.has(id)) missing(errors, String(projection.id), "entity_ids", id);
    for (const id of (projection.relationship_ids ?? []) as string[]) if (!relationships.has(id)) missing(errors, String(projection.id), "relationship_ids", id);
  }
  for (const contract of (snapshot.payload_contracts ?? []) as PayloadContract[]) {
    for (const id of [...((contract.producer_ids ?? []) as string[]), ...((contract.consumer_ids ?? []) as string[])]) {
      if (!entities.has(id)) missing(errors, String(contract.name), "entity_ids", id);
    }
  }
  for (const module of (snapshot.modules ?? []) as LogicalModule[]) {
    for (const id of (module.file_ids ?? []) as string[]) if (!files.has(id)) missing(errors, String(module.id), "file_ids", id);
    for (const id of (module.entity_ids ?? []) as string[]) if (!entities.has(id)) missing(errors, String(module.id), "entity_ids", id);
  }
  for (const finding of (snapshot.findings ?? []) as Finding[]) {
    const message = String(finding.message);
    if (finding.kind === "entity" && !entities.has(String(finding.entity_id))) {
      missing(errors, `finding:${message}`, "entity_id", String(finding.entity_id));
    }
    if (finding.kind === "relationship" && !relationships.has(String(finding.relationship_id))) {
      missing(errors, `finding:${message}`, "relationship_id", String(finding.relationship_id));
    }
    if (finding.kind === "rule" && hierarchy) {
      const target = finding.navigation_target as { scope_id: string };
      if (!hierarchy.scopes.has(target.scope_id as import("../../types/v2").ScopeId))
        missing(errors, `finding:${message}`, "navigation_target.scope_id", target.scope_id);
    }
  }
  return errors.length === 0 ? { ok: true, value: snapshot } : { ok: false, error: errors };
}
