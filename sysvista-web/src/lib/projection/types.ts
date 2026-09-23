import type { CodeEntity, EntityId, FileId, ModuleId, Relationship, RelationshipId, ScopeChild, ScopeId, Snapshot } from "../../types/v2";

export interface ScopeSlice {
  scope_id: ScopeId;
  child_scope_ids?: ScopeId[];
  children?: ScopeChild[];
  child_ids: Array<EntityId | FileId>;
  owner_map: Record<string, EntityId>;
  crossing_relationship_ids: RelationshipId[];
}

export interface ScopeIndex { scopes: ScopeSlice[] }

export type VisibleItem =
  | { kind: "directory"; id: ScopeId; scopeId: ScopeId; name: string }
  | { kind: "file"; id: FileId; scopeId: ScopeId; name: string }
  | { kind: "module"; id: ModuleId; scopeId: ScopeId; name: string }
  | { kind: "symbol"; id: EntityId; scopeId?: ScopeId; name: string; entity?: CodeEntity };

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
  externalTargetId: string;
}

export interface HiddenCounts { entities: number; relationships: number }

export interface ProjectedScope {
  scopeId: ScopeId;
  children: VisibleItem[];
  ownerByEntity: Map<EntityId, string>;
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
