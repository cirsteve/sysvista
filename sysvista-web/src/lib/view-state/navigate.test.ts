import { describe, expect, it } from "vitest";
import fixture from "../../test/fixtures/projection/root-with-three-scopes.json";
import type { ScopeId, Snapshot } from "../../types/v2";
import { buildHierarchyIndex } from "../hierarchy/index";
import { defaultViewState } from "./types";
import { navigateFinding, navigateScope, restoreView } from "./navigate";

describe("view transitions", () => {
  const snapshot = fixture.snapshot as unknown as Snapshot;
  const index = buildHierarchyIndex(snapshot);
  const root = index.rootScopeId;
  const child = index.itemsOf(root).find((item) => item.scope_id)?.scope_id as ScopeId;
  const current = { ...defaultViewState("snapshot", root), viewport: { x: 8, y: 9, zoom: 2 } };

  it("resets the viewport for a fresh descent and retains it on restore", () => {
    expect(navigateScope(current, child).viewportIntent).toBe("fresh");
    expect(navigateScope(current, child).state.viewport).toEqual({ x: 0, y: 0, zoom: 1 });
    expect(navigateScope(current, child, "restore").state.viewport).toEqual(current.viewport);
    expect(restoreView(current).viewportIntent).toBe("restore");
  });

  it("opens the closest valid ancestor and reports a missing finding scope", () => {
    const missing = "scope:missing" as ScopeId;
    const withParent = { ...index, nearestValidScope: (id: ScopeId) => id === missing ? child : index.nearestValidScope(id) };
    const result = navigateFinding(current, withParent, missing, ["entity-1"]);
    expect(result.state.scopeId).toBe(child);
    expect(result.diagnostic?.message).toContain("unavailable");
  });
});
