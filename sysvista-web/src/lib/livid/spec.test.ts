import { describe, expect, it } from "vitest";
import { fixtureProjection } from "./fake";
import { toDiagramSpec } from "./spec";

const GOLDEN = {
  nodes: [
    { id: "a", presentation: "symbol", deferredChildKey: "scope:a-scope" },
    { id: "b", presentation: "symbol", deferredChildKey: "scope:b-scope" },
    { id: "c", presentation: "symbol", deferredChildKey: undefined },
    { id: "proj:root:outside", presentation: "boundary", deferredChildKey: undefined },
  ],
  edges: [
    { source: "a", target: "proj:root:outside", label: "references", count: 1, origin: "heuristic" },
    { source: "a", target: "b", label: "calls", count: 2, origin: "analyzer" },
  ],
};

describe("toDiagramSpec", () => {
  it("structurally matches the root-with-three-scopes golden", () => {
    const spec = toDiagramSpec(fixtureProjection());
    expect({
      nodes: spec.nodes.map(({ id, presentation, deferredChildKey }) => ({ id, presentation, deferredChildKey })),
      edges: spec.edges.map(({ source, target, label, details }) => ({
        source, target, label, count: details.count, origin: details.origin,
      })),
    }).toEqual(GOLDEN);
  });
});
