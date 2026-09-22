import { describe, expect, it } from "vitest";
import type { ScopeId } from "../../types/v2";
import { decodeViewState, encodeViewState } from "./hash";
import { defaultViewState, type ViewState } from "./types";

describe("view-state hash", () => {
  const state: ViewState = {
    snapshotId: "snapshot-7", scopeId: "scope:child" as ScopeId,
    filters: { kinds: ["module", "symbol"], origins: ["analyzer"], query: "router" },
    selection: "entity-3", selectedEntities: ["entity-3" as never], lens: "flow", flowHops: 4,
    viewport: { x: 12.5, y: -4, zoom: 1.75 },
  };

  it("round-trips every field", () => {
    expect(decodeViewState(encodeViewState(state))).toEqual({ state, diagnostics: [] });
  });

  it("rejects malformed input into defaults with a diagnostic", () => {
    const defaults = defaultViewState("current", "root" as ScopeId);
    const decoded = decodeViewState("#sv=v1.%7Bbroken", defaults);
    expect(decoded.state).toEqual(defaults);
    expect(decoded.diagnostics[0]?.kind).toBe("warning");
  });
});
