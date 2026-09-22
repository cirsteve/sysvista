import { describe, expect, it } from "vitest";
import fixture from "../../test/fixtures/projection/root-with-three-scopes.json";
import type { ScopeId, Snapshot } from "../../types/v2";
import { projectScope } from "./project";
import type { ScopeIndex } from "./types";
import { createSliceLoader } from "./slices";

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
  it("reports hidden counts", () => { expect(projected.hidden.entities).toBeGreaterThan(0); });

  it("warn-benchmarks a warm 250k relationship scope", () => {
    const snapshot = fixture.snapshot as unknown as Snapshot;
    const base = snapshot.relationships ?? [];
    const synthetic = { ...snapshot, relationships: Array.from({ length: 250_000 }, (_, index) => ({ ...base[index % base.length], id: `synthetic-${index}` })) } as Snapshot;
    projectScope(synthetic, fixture.index as unknown as ScopeIndex, "root" as ScopeId);
    const start = performance.now();
    projectScope(synthetic, fixture.index as unknown as ScopeIndex, "root" as ScopeId);
    const elapsed = performance.now() - start;
    if (elapsed > 300) console.warn(`projectScope warm 250k benchmark: ${elapsed.toFixed(1)}ms (target 300ms)`);
    else console.info(`projectScope warm 250k benchmark: ${elapsed.toFixed(1)}ms`);
    expect(elapsed).toBeGreaterThanOrEqual(0);
  }, 10_000);

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
