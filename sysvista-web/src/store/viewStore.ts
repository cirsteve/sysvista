import { create } from "zustand";
import type { Diagnostic, ScopeId } from "../types/v2";
import { defaultViewState, type ViewState } from "../lib/view-state/types";

interface ViewStore {
  view: ViewState;
  diagnostics: Diagnostic[];
  theme: "light" | "dark";
  replaceView: (view: ViewState, record?: boolean) => void;
  updateView: (update: Partial<ViewState>, record?: boolean) => void;
  navigateScope: (scopeId: ScopeId, selection?: string | null) => void;
  addDiagnostic: (diagnostic: Diagnostic) => void;
  toggleTheme: () => void;
}

export const useViewStore = create<ViewStore>((set) => ({
  view: defaultViewState(),
  diagnostics: [],
  theme: "light",
  replaceView: (view) => set({ view }),
  updateView: (update) => set((current) => ({ view: { ...current.view, ...update } })),
  navigateScope: (scopeId, selection = null) => set((current) => ({
    view: { ...current.view, scopeId, selection, selectedEntities: [], viewport: { x: 0, y: 0, zoom: 1 } },
  })),
  addDiagnostic: (diagnostic) => set((current) => ({ diagnostics: [...current.diagnostics, diagnostic] })),
  toggleTheme: () => set((current) => ({ theme: current.theme === "light" ? "dark" : "light" })),
}));
