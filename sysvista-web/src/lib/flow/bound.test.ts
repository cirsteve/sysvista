import { describe, expect, it } from "vitest";
import { boundFlowGraph } from "./bound";
import { expandFlow } from "./expand";
import type { FlowGraph } from "./types";

const graph: FlowGraph = {
  horizon: 2, rootIds: ["a" as never],
  nodes: ["d", "b", "a", "c"].map((id, index) => ({
    id: id as never, label: id, depth: index === 2 ? 0 : 1,
    branch: false, truncationCount: 0, unknownContinuations: [],
  })),
  edges: ["b", "c", "d"].map((id) => ({ id: `a-${id}`, source: "a" as never,
    target: id as never, backEdge: false, argumentPayloads: [], returnPayloads: [] })),
};

describe("flow bounds", () => {
  it("truncates breadth and total nodes deterministically", () => {
    const bounded = boundFlowGraph(graph, 2, 3);
    expect(bounded.nodes.map(({ id }) => id)).toEqual(["a", "b", "c"]);
    expect(bounded.nodes[0].truncationCount).toBe(1);
    expect(bounded.edges).toHaveLength(2);
  });
  it("rejects non-finite hops", () => {
    expect(() => boundFlowGraph({ ...graph, horizon: Infinity })).toThrow("finite");
    expect(() => expandFlow({ entities: [] } as never, { kind: "entity", entityId: "a" as never }, NaN)).toThrow("finite");
  });
});
