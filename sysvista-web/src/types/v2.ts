import type * as Generated from "./v2.generated";

declare const idBrand: unique symbol;
type Brand<T extends string> = string & { readonly [idBrand]: T };

export type FileId = Brand<"FileId">;
export type EntityId = Brand<"EntityId">;
export type ModuleId = Brand<"ModuleId">;
export type RelationshipId = Brand<"RelationshipId">;
export type ScopeId = Brand<"ScopeId">;

export type AnalysisStatus = Generated.AnalysisStatus;
export type Diagnostic = Generated.Diagnostic;
export type Evidence = Generated.Evidence;
export type InventoryCounts = Generated.InventoryCounts;
export type Manifest = Generated.Manifest;

export type SourceSpan = Omit<Generated.SourceSpan, "file_id"> & {
  file_id: FileId;
};

export type SourceFile = Omit<Generated.SourceFile, "id"> & {
  id: FileId;
};

export type CodeEntity = Omit<
  Generated.CodeEntity,
  "file_id" | "id" | "owner_id" | "scope_id" | "span"
> & {
  file_id: FileId;
  id: EntityId;
  owner_id?: EntityId | null;
  scope_id: ScopeId;
  span: SourceSpan;
};

export type LogicalModule = Omit<
  Generated.LogicalModule,
  "entity_ids" | "file_ids" | "id" | "scope_id"
> & {
  entity_ids?: EntityId[];
  file_ids?: FileId[];
  id: ModuleId;
  scope_id: ScopeId;
};

type BrandedRelationship<T> = T extends Generated.Relationship
  ? Omit<T, "id" | "source" | "target"> & {
      id: RelationshipId;
      source: EntityId;
      target: EntityId;
    }
  : never;

export type Relationship = BrandedRelationship<Generated.Relationship>;

export type UnresolvedReference = Omit<
  Generated.UnresolvedReference,
  "source" | "span"
> & {
  source: EntityId;
  span: SourceSpan;
};

export interface HeuristicTraversalClaimObject {
  name: string;
  entity_ids: EntityId[];
  relationship_ids: RelationshipId[];
}

export interface Claim {
  evidence_ids?: string[];
  id: string;
  object: HeuristicTraversalClaimObject;
  predicate: "HeuristicTraversal";
  subject: EntityId;
}

export type PayloadContract = Omit<
  Generated.PayloadContract,
  "consumer_ids" | "producer_ids"
> & {
  consumer_ids?: EntityId[];
  producer_ids?: EntityId[];
};

export type Projection = Omit<
  Generated.Projection,
  "entity_ids" | "relationship_ids" | "scope_id"
> & {
  entity_ids?: EntityId[];
  relationship_ids?: RelationshipId[];
  scope_id: ScopeId;
};

type BrandedFinding<T> = T extends { kind: "entity"; entity_id: string }
  ? Omit<T, "entity_id"> & { entity_id: EntityId }
  : T extends { kind: "relationship"; relationship_id: string }
    ? Omit<T, "relationship_id"> & { relationship_id: RelationshipId }
    : T;

export type Finding = BrandedFinding<Generated.Finding>;

export type Snapshot = Omit<
  Generated.Snapshot,
  | "claims"
  | "entities"
  | "findings"
  | "modules"
  | "payload_contracts"
  | "projections"
  | "relationships"
  | "source_files"
  | "unresolved_references"
> & {
  claims?: Claim[];
  entities?: CodeEntity[];
  findings?: Finding[];
  modules?: LogicalModule[];
  payload_contracts?: PayloadContract[];
  projections?: Projection[];
  relationships?: Relationship[];
  source_files?: SourceFile[];
  unresolved_references?: UnresolvedReference[];
};
