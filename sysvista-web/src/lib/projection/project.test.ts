import { describe, expect, it } from "vitest";
import fixture from "../../test/fixtures/projection/root-with-three-scopes.json";
import type { ScopeId, Snapshot } from "../../types/v2";
import { projectScope } from "./project";
import type { ScopeIndex } from "./types";
import { createSliceLoader } from "./slices";
if (process.env.SYSVISTA_RUN_PROJECTION_BENCHMARK === "1") {
  await import("./project.bench");
}

describe("projectScope", () => {
  const projected = projectScope(fixture.snapshot as unknown as Snapshot, fixture.index as unknown as ScopeIndex, "root" as ScopeId);
  it("composes three children and aggregates crossing relationships", () => {
    expect(projected.children.map(({ id }) => id)).toEqual(["a", "b", "c"]);
    expect(projected.relationships.find(({ source, target, kind }) => source === "a" && target === "b" && kind === "calls")?.relationshipIds).toEqual(["r-ab-1", "r-ab-2"]);
  });
  it("summarizes internal relationships per child", () => {
    expect(projected.internalRelationships).toContainEqual({ ownerId: "a", count: 1, byKind: { calls: 1 } });
  });
  it("namespaces boundary nodes and retains the external target", () => {
    expect(projected.boundaryNodes).toContainEqual({ id: "proj:root:outside", kind: "boundary", name: "outside", externalTargetId: "outside" });
  });
  it("does not count summarized internal relationships as hidden", () => {
    expect(projected.hidden.entities).toBeGreaterThan(0);
    expect(projected.hidden.relationships).toBe(0);
  });

  it("does not expose unrelated relationships for a scope with no crossings", () => {
    const index = { scopes: [{ scope_id: "empty", child_ids: [], owner_map: {}, crossing_relationship_ids: [] }] } as unknown as ScopeIndex;
    const result = projectScope(fixture.snapshot as unknown as Snapshot, index, "empty" as ScopeId);
    expect(result.relationships).toEqual([]);
    expect(result.boundaryNodes).toEqual([]);
  });

  it("memoizes slices by snapshot and scope", async () => {
    let reads = 0;
    const load = createSliceLoader({
      readScope: async () => {
        reads += 1;
        return (fixture.index as unknown as ScopeIndex).scopes[0];
      },
      readEntities: async () => [], readRelationships: async () => [],
      readManifest: async () => fixture.snapshot.manifest,
    });
    const first = load("snapshot-1", "root");
    const second = load("snapshot-1", "root");
    expect(first).toBe(second);
    await first;
    expect(reads).toBe(1);
  });
});
