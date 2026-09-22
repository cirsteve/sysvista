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

  it("checks traversal, module, and finding references", () => {
    const snapshot = {
      manifest: {} as Snapshot["manifest"],
      claims: [{ id: "claim", predicate: "HeuristicTraversal", subject: "missing-subject", object: { name: "flow", entity_ids: ["missing-entity"], relationship_ids: ["missing-relationship"] } }],
      modules: [{ id: "module", name: "module", scope_id: "scope", file_ids: ["missing-file"], entity_ids: ["missing-module-entity"] }],
      findings: [
        { kind: "entity", entity_id: "missing-finding-entity", message: "entity finding" },
        { kind: "relationship", relationship_id: "missing-finding-relationship", message: "relationship finding" },
      ],
    } as unknown as Snapshot;
    const result = validateReferences(snapshot);
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.map(({ field }) => field)).toEqual(expect.arrayContaining([
        "subject", "object.entity_ids", "object.relationship_ids", "file_ids", "entity_ids", "entity_id", "relationship_id",
      ]));
    }
  });
});
