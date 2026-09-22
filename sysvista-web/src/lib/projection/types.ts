import type { CodeEntity, EntityId, Relationship, RelationshipId, ScopeId, Snapshot } from "../../types/v2";

export interface ScopeSlice {
  scope_id: ScopeId;
  child_ids: string[];
  owner_map: Record<string, string>;
  crossing_relationship_ids: RelationshipId[];
}

export interface ScopeIndex { scopes: ScopeSlice[] }

export interface AggregateRelationship {
  id: string;
  source: string;
  target: string;
  kind: Relationship["kind"];
  origin: string;
  relationshipIds: RelationshipId[];
}

export interface InternalRelationshipSummary {
  ownerId: string;
  count: number;
  byKind: Record<string, number>;
}

export interface BoundaryNode {
  id: string;
  kind: "boundary";
  name: string;
  externalTargetId: EntityId;
}

export interface HiddenCounts { entities: number; relationships: number }

export interface ProjectedScope {
  scopeId: ScopeId;
  children: CodeEntity[];
  ownerByEntity: Map<string, string>;
  relationships: AggregateRelationship[];
  internalRelationships: InternalRelationshipSummary[];
  boundaryNodes: BoundaryNode[];
  hidden: HiddenCounts;
}

export interface SnapshotSlice {
  snapshotId: string;
  scope: ScopeSlice;
  snapshot: Snapshot;
}
