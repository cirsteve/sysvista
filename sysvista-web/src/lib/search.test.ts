import { describe, expect, it } from "vitest";
import fixture from "../test/fixtures/projection/root-with-three-scopes.json";
import type { Snapshot } from "../types/v2";
import { searchSnapshot } from "./search";

describe("search ranking", () => {
  it("ranks exact, prefix, substring, then path before applying the limit", () => {
    const snapshot = structuredClone(fixture.snapshot) as unknown as Snapshot;
    const template = snapshot.entities![0]!;
    snapshot.entities = [
      { ...template, id: "path" as never, name: "other", qualified_name: "other", file_id: "path-file" as never },
      { ...template, id: "substring" as never, name: "getTarget", qualified_name: "getTarget" },
      { ...template, id: "prefix" as never, name: "targetHandler", qualified_name: "targetHandler" },
      { ...template, id: "exact" as never, name: "target", qualified_name: "target" },
    ];
    snapshot.source_files = [...snapshot.source_files!, { ...snapshot.source_files![0]!, id: "path-file" as never, path: "src/target.ts" }];
    expect(searchSnapshot(snapshot, "target", 3).map((hit) => hit.entityId)).toEqual(["exact", "prefix", "substring"]);
    expect(searchSnapshot(snapshot, "target", 4).map((hit) => hit.entityId)[3]).toBe("path");
  });
});
