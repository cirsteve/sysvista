import {
  defineRegistry,
  formatError,
  layout,
  normalize,
  validateDiagram,
  type DiagramSpec as LividDiagramSpec,
  type StandardSchemaV1,
} from "@rankonelabs/livid-core";
import type { Diagnostic, ScopeId } from "../../types/v2";
import { PRESENTATION_TYPES } from "./fake";
import { toDiagramSpec } from "./spec";
import type {
  AggregateEdgeDetails,
  BoundaryDetails,
  DiagramSpec,
  FileDetails,
  ModuleDetails,
  ScopeProjection,
  ScopeRenderer,
  ScopeRendererListener,
  SymbolDetails,
  Viewport,
} from "./types";

function objectSchema<T extends object>(required: readonly (keyof T)[]): StandardSchemaV1<unknown, T> {
  return {
    "~standard": {
      version: 1,
      vendor: "sysvista",
      types: undefined as unknown as { input: unknown; output: T },
      validate(value) {
        if (typeof value !== "object" || value === null) {
          return { issues: [{ message: "Expected an object" }] };
        }
        const record = value as Record<PropertyKey, unknown>;
        const missing = required.filter((key) => !(key in record));
        return missing.length > 0
          ? { issues: missing.map((key) => ({ message: "Required", path: [key] })) }
          : { value: value as T };
      },
    },
  };
}

const registry = defineRegistry({
  nodeTypes: {
    module: { label: "Module", detail: objectSchema<ModuleDetails>(["name"]), shape: "rounded", glyph: "square", isRouter: false },
    file: { label: "File", detail: objectSchema<FileDetails>(["path"]), shape: "rect", glyph: "bar", isRouter: false },
    symbol: { label: "Symbol", detail: objectSchema<SymbolDetails>(["qualifiedName"]), shape: "stadium", glyph: "dot", isRouter: false },
    boundary: { label: "Boundary", detail: objectSchema<BoundaryDetails>(["externalTargetId"]), shape: "hexagon", glyph: "chevron", isRouter: false },
  },
  edgeTypes: {
    "aggregate-edge": { label: "Dependency", detail: objectSchema<AggregateEdgeDetails>(["count", "origin"]) },
  },
});

const diagnostic = (stage: string, messages: readonly string[]): Diagnostic => ({
  kind: "warning",
  message: `Livid ${stage} failed; showing the fixture-compatible surface (${messages.join("; ")})`,
});

function asLividSpec(spec: DiagramSpec): LividDiagramSpec {
  return {
    profile: "dependency",
    nodes: spec.nodes.map((node) => ({
      id: node.id,
      type: node.presentation,
      label: node.label,
      detail: node.details,
      ...(node.deferredChildKey
        ? { childState: { kind: "deferred" as const, key: node.deferredChildKey } }
        : {}),
    })),
    edges: spec.edges.map((edge) => ({
      id: edge.id,
      type: edge.presentation,
      source: edge.source,
      target: edge.target,
      label: edge.label,
      detail: edge.details,
    })),
  };
}

export interface RendererLoadResult { renderer: ScopeRenderer; diagnostic?: Diagnostic }

export class LividScopeRenderer implements ScopeRenderer {
  private readonly listeners = new Set<ScopeRendererListener>();

  registerPresentationTypes() { return PRESENTATION_TYPES; }
  toDiagramSpec(projection: ScopeProjection) { return toDiagramSpec(projection); }

  async render(projection: ScopeProjection) {
    const spec = this.toDiagramSpec(projection);
    const valid = validateDiagram(registry, asLividSpec(spec));
    if (!valid.ok) {
      return { spec, diagram: null, diagnostic: diagnostic("validation", valid.error.map(formatError)) };
    }
    const normalized = normalize(valid.value);
    if (!normalized.ok) {
      return { spec, diagram: null, diagnostic: diagnostic("normalization", normalized.error.map(formatError)) };
    }
    const laidOut = await layout(normalized.value);
    if (!laidOut.ok) {
      return { spec, diagram: null, diagnostic: diagnostic("layout", laidOut.error.map(formatError)) };
    }
    return { spec, diagram: laidOut.value };
  }

  subscribe(listener: ScopeRendererListener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
  private emit(event: Parameters<ScopeRendererListener>[0]) { this.listeners.forEach((listener) => listener(event)); }
  select(id: string | null) { this.emit({ type: "select", id }); }
  descend(scopeId: ScopeId, deferredChildKey: string) { this.emit({ type: "descend", scopeId, deferredChildKey }); }
  focus(id: string, viewport?: Viewport) { this.emit({ type: "focus", id, ...(viewport && { viewport }) }); }
  replace(snapshotId: string, scopeId: ScopeId, spec: DiagramSpec) { this.emit({ type: "replace", snapshotId, scopeId, spec }); }
}

/** Static imports make missing/incompatible Livid packages a build-time failure. */
export async function loadScopeRenderer(): Promise<RendererLoadResult> {
  const renderer = new LividScopeRenderer();
  renderer.registerPresentationTypes();
  return { renderer };
}
