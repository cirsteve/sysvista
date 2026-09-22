import { create } from "zustand";
import type { Diagnostic, ScopeId } from "../types/v2";
import { createHistory, historyBack, historyForward, pushHistory, type ViewHistory } from "../lib/view-state/history";
import { defaultViewState, type ViewState } from "../lib/view-state/types";

interface ViewStore {
  view: ViewState;
  history: ViewHistory;
  diagnostics: Diagnostic[];
  theme: "light" | "dark";
  replaceView: (view: ViewState, record?: boolean) => void;
  updateView: (update: Partial<ViewState>, record?: boolean) => void;
  navigateScope: (scopeId: ScopeId, selection?: string | null) => void;
  back: () => void;
  forward: () => void;
  addDiagnostic: (diagnostic: Diagnostic) => void;
  toggleTheme: () => void;
}

const initial = defaultViewState();

export const useViewStore = create<ViewStore>((set) => ({
  view: initial,
  history: createHistory(initial),
  diagnostics: [],
  theme: "light",
  replaceView: (view, record = true) => set((current) => ({
    view,
    history: record ? pushHistory(current.history, view) : current.history,
  })),
  updateView: (update, record = true) => set((current) => {
    const view = { ...current.view, ...update };
    return { view, history: record ? pushHistory(current.history, view) : current.history };
  }),
  navigateScope: (scopeId, selection = null) => set((current) => {
    const view = { ...current.view, scopeId, selection };
    return { view, history: pushHistory(current.history, view) };
  }),
  back: () => set((current) => {
    const [history, view] = historyBack(current.history);
    return { history, view };
  }),
  forward: () => set((current) => {
    const [history, view] = historyForward(current.history);
    return { history, view };
  }),
  addDiagnostic: (diagnostic) => set((current) => ({ diagnostics: [...current.diagnostics, diagnostic] })),
  toggleTheme: () => set((current) => ({ theme: current.theme === "light" ? "dark" : "light" })),
}));
