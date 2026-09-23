import type {
  EntityId,
  FileId,
  ScopeId,
  Snapshot,
  SourceFile,
  SourceSpan,
} from "../../types/v2";
import type { ProjectedScope, ScopeIndex } from "../projection/types";

export type PresentationType = "directory" | "module" | "file" | "symbol" | "boundary" | "flow" | "aggregate-edge" | "flow-edge";

export interface DetailField {
  key: string;
  label: string;
  type: "text" | "number" | "code" | "evidence";
  required?: boolean;
}

export interface PresentationRegistration {
  type: PresentationType;
  label: string;
  detailSchema: readonly DetailField[];
}

interface DiagramNodeBase<P extends Exclude<PresentationType, "aggregate-edge" | "flow-edge">, D> {
  id: string;
  presentation: P;
  label: string;
  deferredChildKey?: string;
  details: D;
}

export interface ModuleDetails {
  name: string;
  fileIds: string[];
  entityIds: string[];
}
export interface DirectoryDetails { name: string }

export interface FileDetails {
  path: string;
  language: string;
  analysis: SourceFile["analysis"];
}

export interface SymbolDetails {
  name: string;
  qualifiedName: string;
  declarationKind: string;
  fileId: FileId;
  span: SourceSpan;
}

export interface BoundaryDetails { externalTargetId: string }
export interface FlowDetails {
  depth: number;
  branch: boolean;
  truncationCount: number;
  unknownContinuations: { id: string; name: string; reason: string }[];
}

export type DiagramNode =
  | DiagramNodeBase<"directory", DirectoryDetails>
  | DiagramNodeBase<"module", ModuleDetails>
  | DiagramNodeBase<"file", FileDetails>
  | DiagramNodeBase<"symbol", SymbolDetails>
  | DiagramNodeBase<"boundary", BoundaryDetails>
  | DiagramNodeBase<"flow", FlowDetails>;

export interface AggregateEdgeDetails {
  kind: string;
  origin: string;
  origins?: string[];
  count: number;
  relationshipIds: string[];
}

export interface AggregateDiagramEdge {
  id: string;
  presentation: "aggregate-edge";
  source: string;
  target: string;
  label: string;
  details: AggregateEdgeDetails;
}

export interface FlowDiagramEdge {
  id: string;
  presentation: "flow-edge";
  source: string;
  target: string;
  label: string;
  details: {
    backEdge: boolean;
    argumentPayloads: string[];
    returnPayloads: string[];
  };
}

export type DiagramEdge = AggregateDiagramEdge | FlowDiagramEdge;

export interface DiagramSpec {
  id: string;
  scopeId: ScopeId;
  semanticsProfile: string;
  nodes: DiagramNode[];
  edges: DiagramEdge[];
  overBudget?: true;
  items?: import("../projection/types").VisibleItem[];
}

export interface ScopeProjection {
  snapshot: Snapshot;
  index: ScopeIndex;
  projected: ProjectedScope;
}

/** Opaque outside lib/livid: only the adapter constructs this Livid render model. */
export type RenderedDiagram = object;

export interface ScopeRenderResult {
  spec: DiagramSpec;
  diagram: RenderedDiagram | null;
  diagnostic?: import("../../types/v2").Diagnostic;
}

export type ScopeRendererEvent =
  | { type: "select"; id: EntityId | FileId | string | null }
  | { type: "descend"; scopeId: ScopeId; deferredChildKey: string }
  | { type: "focus"; id: EntityId | FileId | string; viewport?: Viewport }
  | { type: "replace"; snapshotId: string; scopeId: ScopeId; spec: DiagramSpec };

export interface Viewport {
  x: number;
  y: number;
  zoom: number;
}

export type ScopeRendererListener = (event: ScopeRendererEvent) => void;

/**
 * Stable rendering boundary owned by SysVista. Implementations may use Livid,
 * SVG, or a test fake; consumers never need to import implementation types.
 */
export interface ScopeRenderer {
  registerPresentationTypes(): readonly PresentationRegistration[];
  toDiagramSpec(projection: ScopeProjection): DiagramSpec;
  render(projection: ScopeProjection): Promise<ScopeRenderResult>;
  renderSpec(spec: DiagramSpec): Promise<ScopeRenderResult>;
  subscribe(listener: ScopeRendererListener): () => void;
  select(id: EntityId | FileId | string | null): void;
  descend(scopeId: ScopeId, deferredChildKey: string): void;
  focus(id: EntityId | FileId | string, viewport?: Viewport): void;
  replace(snapshotId: string, scopeId: ScopeId, spec: DiagramSpec): void;
}

export const DEPENDENCY_SEMANTICS_PROFILE = "dependency";
export const FLOW_SEMANTICS_PROFILE = "flow";

export const deferredChildKey = (scopeId: ScopeId): string => `scope:${scopeId}`;
