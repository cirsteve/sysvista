import { describe, expect, it } from "vitest";
import type { Snapshot } from "../../types/v2";
import { validateReferences } from "./references";

describe("validateReferences", () => {
  it("reports missing child scopes, invisible owners, and containment cycles separately", () => {
    const snapshot = { manifest: { root_scope_id: "root" }, scope_index: { scopes: [
      { scope_id: "root", children: [{ kind: "directory", scope_id: "child" }, { kind: "directory", scope_id: "missing" }], owner_map: { entity: "outside" }, crossing_relationship_ids: [] },
      { scope_id: "child", children: [{ kind: "directory", scope_id: "root" }], owner_map: {}, crossing_relationship_ids: [] },
    ] } } as unknown as Snapshot;
    const result = validateReferences(snapshot);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.error.map(({ field }) => field)).toEqual(expect.arrayContaining([
      "children.scope_id", "owner_map", "containment_cycle",
    ]));
  });
  it("identifies the owner of a dangling relationship endpoint", () => {
    const snapshot = { manifest: {} as Snapshot["manifest"], relationships: [{ id: "r1", kind: "calls", source: "missing-a", target: "missing-b", origin: "test" }] } as unknown as Snapshot;
    const result = validateReferences(snapshot);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.error.map(({ ownerId }) => ownerId)).toEqual(["r1", "r1"]);
  });

  it("rejects typed children and owner-map keys missing from snapshot collections", () => {
    const snapshot = {
      manifest: { root_scope_id: "root" },
      scope_index: { scopes: [
        { scope_id: "root", children: [
          { kind: "file", file_id: "missing-file", scope_id: "file-scope" },
          { kind: "module", module_id: "missing-module", scope_id: "module-scope" },
          { kind: "symbol", entity_id: "missing-symbol", scope_id: null },
        ], owner_map: { "missing-owner": "missing-file" }, crossing_relationship_ids: [] },
        { scope_id: "file-scope", children: [], owner_map: {}, crossing_relationship_ids: [] },
        { scope_id: "module-scope", children: [], owner_map: {}, crossing_relationship_ids: [] },
      ] },
    } as unknown as Snapshot;
    const result = validateReferences(snapshot);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.error.map(({ field }) => field)).toEqual(expect.arrayContaining([
      "children.file_id", "children.module_id", "children.entity_id", "owner_map.entity_id",
    ]));
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
