import type { EntityId, ScopeId } from "../../types/v2";
import type { Viewport } from "../livid/types";

export interface ViewFilters {
  kinds: string[];
  origins: string[];
  query: string;
}

export interface ViewState {
  snapshotId: string;
  scopeId: ScopeId;
  filters: ViewFilters;
  selection: EntityId | string | null;
  viewport: Viewport;
}

export const defaultViewState = (snapshotId = "", scopeId = "scope:root" as ScopeId): ViewState => ({
  snapshotId,
  scopeId,
  filters: { kinds: [], origins: [], query: "" },
  selection: null,
  viewport: { x: 0, y: 0, zoom: 1 },
});
