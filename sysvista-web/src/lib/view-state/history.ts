import type { ViewState } from "./types";

export interface ViewHistory {
  entries: ViewState[];
  cursor: number;
}

const clone = (state: ViewState): ViewState => structuredClone(state);
export const createHistory = (initial: ViewState): ViewHistory => ({ entries: [clone(initial)], cursor: 0 });

export function pushHistory(history: ViewHistory, state: ViewState, limit = 50): ViewHistory {
  const entries = [...history.entries.slice(0, history.cursor + 1), clone(state)].slice(-limit);
  return { entries, cursor: entries.length - 1 };
}

export const historyBack = (history: ViewHistory): [ViewHistory, ViewState] => {
  const cursor = Math.max(0, history.cursor - 1);
  return [{ ...history, cursor }, clone(history.entries[cursor])];
};

export const historyForward = (history: ViewHistory): [ViewHistory, ViewState] => {
  const cursor = Math.min(history.entries.length - 1, history.cursor + 1);
  return [{ ...history, cursor }, clone(history.entries[cursor])];
};
