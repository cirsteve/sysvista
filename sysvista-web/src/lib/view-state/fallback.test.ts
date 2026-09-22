import { describe, expect, it } from "vitest";
import type { ScopeId } from "../../types/v2";
import { nearestValidScope } from "./fallback";

describe("nearestValidScope", () => {
  it("resolves a missing scope to its nearest valid ancestor and records why", () => {
    const id = (value: string) => value as ScopeId;
    const result = nearestValidScope(id("symbol"), new Set([id("root"), id("file")]), new Map([[id("symbol"), id("file")], [id("file"), id("root")]]), id("root"));
    expect(result.scopeId).toBe("file");
    expect(result.diagnostic?.message).toContain("nearest ancestor");
  });
});
