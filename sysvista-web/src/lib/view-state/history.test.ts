import { describe, expect, it } from "vitest";
import type { ScopeId } from "../../types/v2";
import { createHistory, historyBack, historyForward, pushHistory } from "./history";
import { defaultViewState } from "./types";

describe("view history", () => {
  it("back then forward restores the identical full state", () => {
    const first = defaultViewState("one", "root" as ScopeId);
    const second = { ...first, scopeId: "child" as ScopeId, selection: "symbol", viewport: { x: 5, y: 6, zoom: 2 }, filters: { kinds: ["symbol"], origins: ["source"], query: "x" } };
    const pushed = pushHistory(createHistory(first), second);
    const [back] = historyBack(pushed);
    const [, restored] = historyForward(back);
    expect(restored).toEqual(second);
  });

  it("caps entries at fifty", () => {
    const initial = defaultViewState();
    const history = Array.from({ length: 60 }, (_, index) => index).reduce(
      (current, index) => pushHistory(current, { ...initial, snapshotId: String(index) }),
      createHistory(initial),
    );
    expect(history.entries).toHaveLength(50);
  });
});
