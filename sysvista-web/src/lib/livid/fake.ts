import fixture from "../../test/fixtures/projection/root-with-three-scopes.json";
import type { ScopeId, Snapshot } from "../../types/v2";
import { projectScope } from "../projection/project";
import type { ScopeIndex } from "../projection/types";
import { toDiagramSpec } from "./spec";
import type {
  DiagramSpec,
  PresentationRegistration,
  ScopeProjection,
  ScopeRenderer,
  ScopeRendererEvent,
  ScopeRendererListener,
  Viewport,
} from "./types";

export const PRESENTATION_TYPES: readonly PresentationRegistration[] = [
  { type: "module", label: "Module", detailSchema: [{ key: "name", label: "Name", type: "text", required: true }] },
  { type: "file", label: "File", detailSchema: [{ key: "path", label: "Path", type: "code", required: true }] },
  { type: "symbol", label: "Symbol", detailSchema: [{ key: "qualifiedName", label: "Qualified name", type: "code", required: true }] },
  { type: "boundary", label: "Boundary", detailSchema: [{ key: "externalTargetId", label: "External target", type: "code", required: true }] },
  { type: "aggregate-edge", label: "Dependency", detailSchema: [
    { key: "count", label: "Evidence count", type: "number", required: true },
    { key: "origin", label: "Evidence origin", type: "evidence", required: true },
  ] },
];

export class FakeScopeRenderer implements ScopeRenderer {
  private readonly listeners = new Set<ScopeRendererListener>();

  registerPresentationTypes() { return PRESENTATION_TYPES; }
  toDiagramSpec(projection: ScopeProjection) { return toDiagramSpec(projection); }
  async render(projection: ScopeProjection) {
    return { spec: this.toDiagramSpec(projection), diagram: null };
  }
  subscribe(listener: ScopeRendererListener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
  private emit(event: ScopeRendererEvent) { this.listeners.forEach((listener) => listener(event)); }
  select(id: string | null) { this.emit({ type: "select", id }); }
  descend(scopeId: ScopeId, key: string) { this.emit({ type: "descend", scopeId, deferredChildKey: key }); }
  focus(id: string, viewport?: Viewport) { this.emit({ type: "focus", id, ...(viewport && { viewport }) }); }
  replace(snapshotId: string, scopeId: ScopeId, spec: DiagramSpec) {
    this.emit({ type: "replace", snapshotId, scopeId, spec });
  }
}

export function fixtureProjection(): ScopeProjection {
  const snapshot = fixture.snapshot as unknown as Snapshot;
  const index = fixture.index as unknown as ScopeIndex;
  const scopeId = "root" as ScopeId;
  return { snapshot, index, projected: projectScope(snapshot, index, scopeId) };
}

export function createFixtureRenderer(): FakeScopeRenderer {
  return new FakeScopeRenderer();
}
