import { describe, expect, it } from "vitest";
import { buildHierarchyIndex } from "./index";
import type { Snapshot } from "../../types/v2";

const snapshot = {
  manifest: { root_scope_id: "root" },
  entities: [{ id: "entity", scope_id: "file-scope" }],
  scope_index: { scopes: [
    { scope_id: "root", children: [{ kind: "file", file_id: "file", scope_id: "file-scope" }], owner_map: { entity: "file" }, crossing_relationship_ids: [] },
    { scope_id: "file-scope", children: [{ kind: "symbol", entity_id: "entity" }], owner_map: {}, crossing_relationship_ids: [] },
  ] },
} as unknown as Snapshot;

describe("hierarchy index", () => {
  it("memoizes scope, membership and representative lookups", () => {
    const index = buildHierarchyIndex(snapshot);
    expect(buildHierarchyIndex(snapshot)).toBe(index);
    expect(index.itemsOf("root" as never)).toHaveLength(1);
    expect(index.scopeOf("entity")).toBe("file-scope");
    expect(index.representativeOf("root" as never, "entity" as never)).toBe("file");
    expect(index.nearestValidScope("file-scope" as never)).toBe("file-scope");
  });
});
