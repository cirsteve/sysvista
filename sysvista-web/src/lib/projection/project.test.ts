import { describe, expect, it } from "vitest";
import fixture from "../../test/fixtures/projection/root-with-three-scopes.json";
import type { Relationship, ScopeId, Snapshot } from "../../types/v2";
import { projectScope } from "./project";
import { aggregateCrossingRelationships } from "./aggregate";
import type { ScopeIndex } from "./types";
import { createSliceLoader } from "./slices";
if (process.env.SYSVISTA_RUN_PROJECTION_BENCHMARK === "1") await import("./project.bench");

const snapshot = fixture.snapshot as unknown as Snapshot;
const index = fixture.index as unknown as ScopeIndex;
const root = snapshot.manifest.root_scope_id as ScopeId;

describe("projectScope with CLI fixture", () => {
  it("projects typed physical and logical children", () => {
    const projected = projectScope(snapshot, index, root);
    expect(projected.children.some((item) => item.kind === "directory")).toBe(true);
    expect(projected.children.some((item) => item.kind === "module")).toBe(true);
    const rawCount = index.scopes.find((scope) => scope.scope_id === root)?.children?.length;
    expect(projected.children).toHaveLength(rawCount ?? 0);
  });
  it("maps transitive entities to visible owner ids and retains internal counts", () => {
    const entities = snapshot.entities ?? [];
    const [source, target] = entities;
    const relationship = { id: "internal", kind: "calls", source: source.id, target: target.id, origin: "resolved" } as unknown as Relationship;
    const withRelationship = { ...snapshot, relationships: [relationship] } as Snapshot;
    const withIndex = { scopes: index.scopes.map((scope) => scope.scope_id === root
      ? { ...scope, crossing_relationship_ids: ["internal"] as never }
      : scope) } as ScopeIndex;
    const projected = projectScope(withRelationship, withIndex, root);
    expect(projected.ownerByEntity.get(source.id)).toBeDefined();
    expect(projected.internalRelationships.reduce((sum, item) => sum + item.count, 0)).toBe(1);
  });
  it("gives external endpoints namespaced boundary nodes", () => {
    const source = snapshot.entities![0];
    const relationship = { id: "external", kind: "calls", source: source.id, target: "outside", origin: "resolved" } as unknown as Relationship;
    const withIndex = { scopes: index.scopes.map((scope) => scope.scope_id === root
      ? { ...scope, crossing_relationship_ids: ["external"] as never }
      : scope) } as ScopeIndex;
    const projected = projectScope({ ...snapshot, relationships: [relationship] }, withIndex, root);
    expect(projected.boundaryNodes).toContainEqual(expect.objectContaining({ id: `proj:${root}:outside`, externalTargetId: "outside" }));
  });
  it("uses full aggregate tuples as collision-free keys", () => {
    const relationships = [
      { id: "r1", source: "ab", target: "c", kind: "calls", origin: "analyzer" },
      { id: "r2", source: "a", target: "bc", kind: "calls", origin: "analyzer" },
    ] as unknown as Relationship[];
    const aggregated = aggregateCrossingRelationships(root, relationships, new Map());
    expect(new Set(aggregated.map(({ id }) => id)).size).toBe(2);
  });
  it("does not expose unrelated relationships in a scope with no crossings", () => {
    const empty = { scopes: [{ scope_id: "empty", children: [], child_ids: [], owner_map: {}, crossing_relationship_ids: [] }] } as unknown as ScopeIndex;
    expect(projectScope(snapshot, empty, "empty" as ScopeId).relationships).toEqual([]);
  });
  it("memoizes slices by snapshot and scope", async () => {
    let reads = 0;
    const load = createSliceLoader({
      readScope: async () => { reads += 1; return { ...index.scopes[0], child_ids: [] }; },
      readEntities: async () => [], readRelationships: async () => [],
      readManifest: async () => snapshot.manifest,
    });
    const first = load("snapshot-1", root);
    expect(load("snapshot-1", root)).toBe(first);
    await first;
    expect(reads).toBe(1);
  });
});
