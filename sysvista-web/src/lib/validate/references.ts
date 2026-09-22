import type { Claim, Evidence, PayloadContract, Projection, Snapshot } from "../../types/v2";
import type { Result } from "../result";

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
  const files = new Set((snapshot.source_files ?? []).map(({ id }) => id));
  const entities = new Set((snapshot.entities ?? []).map(({ id }) => id));
  const relationships = new Set((snapshot.relationships ?? []).map(({ id }) => id));
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
  }
  for (const item of snapshot.unresolved_references ?? []) {
    if (!entities.has(item.source)) missing(errors, `unresolved:${item.name}`, "source", item.source);
    if (!files.has(item.span.file_id)) missing(errors, `unresolved:${item.name}`, "span.file_id", item.span.file_id);
  }
  for (const projection of (snapshot.projections ?? []) as Projection[]) {
    for (const id of (projection.entity_ids ?? []) as string[]) if (!entities.has(id as never)) missing(errors, String(projection.id), "entity_ids", id);
    for (const id of (projection.relationship_ids ?? []) as string[]) if (!relationships.has(id as never)) missing(errors, String(projection.id), "relationship_ids", id);
  }
  for (const contract of (snapshot.payload_contracts ?? []) as PayloadContract[]) {
    for (const id of [...((contract.producer_ids ?? []) as string[]), ...((contract.consumer_ids ?? []) as string[])]) {
      if (!entities.has(id as never)) missing(errors, String(contract.name), "entity_ids", id);
    }
  }
  return errors.length === 0 ? { ok: true, value: snapshot } : { ok: false, error: errors };
}
