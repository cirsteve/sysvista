import { describe, expect, it } from "vitest";
import type { Snapshot } from "../../types/v2";
import { validateReferences } from "./references";

describe("validateReferences", () => {
  it("identifies the owner of a dangling relationship endpoint", () => {
    const snapshot = { manifest: {} as Snapshot["manifest"], relationships: [{ id: "r1", kind: "calls", source: "missing-a", target: "missing-b", origin: "test" }] } as unknown as Snapshot;
    const result = validateReferences(snapshot);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.error.map(({ ownerId }) => ownerId)).toEqual(["r1", "r1"]);
  });
});
