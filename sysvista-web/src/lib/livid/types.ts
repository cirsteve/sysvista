import type {
  EntityId,
  FileId,
  ScopeId,
  Snapshot,
} from "../../types/v2";
import type { ProjectedScope, ScopeIndex } from "../projection/types";

export type PresentationType = "module" | "file" | "symbol" | "boundary" | "aggregate-edge";

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

export interface DiagramNode {
  id: string;
  presentation: Exclude<PresentationType, "aggregate-edge">;
  label: string;
  deferredChildKey?: string;
  details: Record<string, unknown>;
}

export interface DiagramEdge {
  id: string;
  presentation: "aggregate-edge";
  source: string;
  target: string;
  label: string;
  details: Record<string, unknown>;
}

export interface DiagramSpec {
  id: string;
  scopeId: ScopeId;
  semanticsProfile: string;
  nodes: DiagramNode[];
  edges: DiagramEdge[];
}

export interface ScopeProjection {
  snapshot: Snapshot;
  index: ScopeIndex;
  projected: ProjectedScope;
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
  subscribe(listener: ScopeRendererListener): () => void;
  select(id: EntityId | FileId | string | null): void;
  descend(scopeId: ScopeId, deferredChildKey: string): void;
  focus(id: EntityId | FileId | string, viewport?: Viewport): void;
  replace(snapshotId: string, scopeId: ScopeId, spec: DiagramSpec): void;
}

export const DEPENDENCY_SEMANTICS_PROFILE = "sysvista-dependency-v1";

export const deferredChildKey = (scopeId: ScopeId): string => `scope:${scopeId}`;
