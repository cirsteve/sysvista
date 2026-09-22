import { describe, expect, it } from "vitest";
import type { ScopeId } from "../../types/v2";
import { ScopeRequestCoordinator, scopeRenderFailureDiagnostic } from "./requests";

describe("ScopeRequestCoordinator", () => {
  it("discards a late response after the full request tuple is superseded", () => {
    const coordinator = new ScopeRequestCoordinator<string>();
    const root = "root" as ScopeId;
    const first = { snapshotId: "snapshot-1", scopeId: root, requestId: "request-1" };
    const second = { snapshotId: "snapshot-1", scopeId: root, requestId: "request-2" };
    const replacements: string[] = [];

    coordinator.begin(first);
    coordinator.begin(second);

    expect(coordinator.commit(first, "late", (value) => replacements.push(value))).toBe(false);
    expect(replacements).toEqual([]);
    expect(coordinator.commit(second, "current", (value) => replacements.push(value))).toBe(true);
    expect(replacements).toEqual(["current"]);
  });

  it("compares snapshot and scope as well as request id", () => {
    const coordinator = new ScopeRequestCoordinator<number>();
    const current = { snapshotId: "two", scopeId: "child" as ScopeId, requestId: "same" };
    coordinator.begin(current);
    expect(coordinator.isCurrent({ ...current, snapshotId: "one" })).toBe(false);
    expect(coordinator.isCurrent({ ...current, scopeId: "root" as ScopeId })).toBe(false);
  });

  it("turns renderer rejection into a visible warning Diagnostic", () => {
    expect(scopeRenderFailureDiagnostic(new Error("layout crashed"))).toEqual({
      kind: "warning",
      message: "Scope rendering failed; showing the fixture-compatible surface (layout crashed)",
    });
  });
});
