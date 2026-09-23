import type { Diagnostic, ScopeId } from "../../types/v2";
import type { HierarchyIndex } from "../hierarchy/index";
import type { ViewState } from "./types";

export type ViewportIntent = "fresh" | "restore";
export interface NavigationResult {
  state: ViewState;
  viewportIntent: ViewportIntent;
  diagnostic?: Diagnostic;
}

export function navigateScope(view: ViewState, scopeId: ScopeId, intent: ViewportIntent = "fresh"): NavigationResult {
  return { viewportIntent: intent, state: { ...view, scopeId, selection: null, selectedEntities: [],
    viewport: intent === "fresh" ? { x: 0, y: 0, zoom: 1 } : view.viewport } };
}

export function restoreView(view: ViewState): NavigationResult {
  return { state: view, viewportIntent: "restore" };
}

export function navigateFinding(view: ViewState, index: HierarchyIndex, scopeId: ScopeId, entityIds: string[]): NavigationResult {
  const valid = index.nearestValidScope(scopeId);
  return {
    state: { ...navigateScope(view, valid).state, selection: entityIds[0] ?? null, selectedEntities: entityIds as ViewState["selectedEntities"] },
    viewportIntent: "fresh",
    ...(valid !== scopeId ? { diagnostic: { kind: "warning" as const, message: `Finding scope ${scopeId} is unavailable; opened ${valid}` } } : {}),
  };
}
