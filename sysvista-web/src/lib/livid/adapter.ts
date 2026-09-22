import type { Diagnostic, ScopeId } from "../../types/v2";
import { FakeScopeRenderer, PRESENTATION_TYPES } from "./fake";
import { toDiagramSpec } from "./spec";
import type { DiagramSpec, ScopeProjection, ScopeRenderer, ScopeRendererListener, Viewport } from "./types";

type UnknownModule = Record<string, unknown>;

export interface RendererLoadResult {
  renderer: ScopeRenderer;
  diagnostic?: Diagnostic;
}

class LividScopeRenderer implements ScopeRenderer {
  private readonly listeners = new Set<ScopeRendererListener>();
  private readonly core: UnknownModule;

  constructor(core: UnknownModule) { this.core = core; }
  registerPresentationTypes() {
    const register = this.core.registerPresentationTypes;
    if (typeof register === "function") register(PRESENTATION_TYPES);
    return PRESENTATION_TYPES;
  }
  toDiagramSpec(projection: ScopeProjection) {
    const spec = toDiagramSpec(projection);
    const validate = this.core.validateDiagramSpec;
    if (typeof validate === "function") validate(spec);
    return spec;
  }
  subscribe(listener: ScopeRendererListener) { this.listeners.add(listener); return () => this.listeners.delete(listener); }
  private emit(event: Parameters<ScopeRendererListener>[0]) { this.listeners.forEach((listener) => listener(event)); }
  select(id: string | null) { this.emit({ type: "select", id }); }
  descend(scopeId: ScopeId, deferredChildKey: string) { this.emit({ type: "descend", scopeId, deferredChildKey }); }
  focus(id: string, viewport?: Viewport) { this.emit({ type: "focus", id, ...(viewport && { viewport }) }); }
  replace(snapshotId: string, scopeId: ScopeId, spec: DiagramSpec) { this.emit({ type: "replace", snapshotId, scopeId, spec }); }
}

/** The sole translation point between the optional Livid packages and SysVista. */
export async function loadScopeRenderer(): Promise<RendererLoadResult> {
  try {
    const packageName = "@rankonelabs/livid-core";
    const core = await import(/* @vite-ignore */ packageName) as UnknownModule;
    const renderer = new LividScopeRenderer(core);
    renderer.registerPresentationTypes();
    return { renderer };
  } catch (cause) {
    const message = cause instanceof Error ? cause.message : "unknown Livid loading error";
    return {
      renderer: new FakeScopeRenderer(),
      diagnostic: {
        kind: "warning",
        message: `Livid validation is unavailable; using the fixture-compatible renderer (${message})`,
      },
    };
  }
}
