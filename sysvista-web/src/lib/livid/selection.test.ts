import { describe, expect, it } from "vitest";
import { resolveSelection, selectionFrame } from "./selection";
import type { DiagramSpec } from "./types";

const spec: DiagramSpec = {
  id: "scope:s", scopeId: "s" as DiagramSpec["scopeId"], semanticsProfile: "dependency",
  nodes: ["a", "b", "c"].map((id) => ({ id, presentation: "directory" as const, label: id, details: { name: id } })),
  edges: [{ id: "aggregate:a-b", presentation: "aggregate-edge", source: "a", target: "b",
    label: "calls", details: { kind: "calls", origin: "analyzer", count: 1, relationshipIds: ["r1"] } }],
};

describe("Livid selection", () => {
  it("resolves nodes, aggregates and underlying relationships", () => {
    expect(resolveSelection(spec, "a")).toMatchObject({ kind: "node", id: "a" });
    expect(resolveSelection(spec, "aggregate:a-b")).toMatchObject({ kind: "edge", id: "aggregate:a-b" });
    expect(resolveSelection(spec, "r1")).toMatchObject({ kind: "edge", id: "aggregate:a-b" });
    expect(resolveSelection(spec, "stale")).toBeNull();
  });
  it("highlights all three finding entities", () => {
    expect(selectionFrame(spec, ["a", "b", "c"]).nodes).toEqual({ a: "highlight", b: "highlight", c: "highlight" });
  });
});
