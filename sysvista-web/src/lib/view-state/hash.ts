import type { Diagnostic, ScopeId } from "../../types/v2";
import { defaultViewState, type ViewState } from "./types";

const PREFIX = "#sv=v1.";

export interface DecodedViewState {
  state: ViewState;
  diagnostics: Diagnostic[];
}

const diagnostic = (message: string): Diagnostic => ({ kind: "warning", message });
const record = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null;

const validState = (value: unknown): value is ViewState => {
  if (!record(value) || !record(value.filters) || !record(value.viewport)) return false;
  return typeof value.snapshotId === "string" && typeof value.scopeId === "string" &&
    (typeof value.selection === "string" || value.selection === null) &&
    Array.isArray(value.filters.kinds) && value.filters.kinds.every((item) => typeof item === "string") &&
    Array.isArray(value.filters.origins) && value.filters.origins.every((item) => typeof item === "string") &&
    typeof value.filters.query === "string" &&
    [value.viewport.x, value.viewport.y, value.viewport.zoom].every((item) => typeof item === "number" && Number.isFinite(item));
};

export const encodeViewState = (state: ViewState): string => `${PREFIX}${encodeURIComponent(JSON.stringify(state))}`;

export function decodeViewState(hash: string, defaults = defaultViewState()): DecodedViewState {
  if (!hash) return { state: defaults, diagnostics: [] };
  if (!hash.startsWith(PREFIX)) {
    return { state: defaults, diagnostics: [diagnostic("Unsupported view-state hash version; defaults restored")] };
  }
  try {
    const value: unknown = JSON.parse(decodeURIComponent(hash.slice(PREFIX.length)));
    if (!validState(value)) throw new Error("view state does not match version 1");
    return { state: { ...value, scopeId: value.scopeId as ScopeId }, diagnostics: [] };
  } catch (cause) {
    const reason = cause instanceof Error ? cause.message : "unknown parse error";
    return { state: defaults, diagnostics: [diagnostic(`Malformed view-state hash; defaults restored (${reason})`)] };
  }
}
