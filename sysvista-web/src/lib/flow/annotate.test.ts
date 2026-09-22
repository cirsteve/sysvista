import { describe, expect, it } from "vitest";
import type { Snapshot } from "../../types/v2";
import { annotatePayloads } from "./annotate";
import { flowPresentation } from "../livid/presentation";
import type { FlowGraph } from "./types";

describe("annotatePayloads", () => {
  const snapshot = { payload_contracts: [{ name: "Request", producer_ids: ["a"], consumer_ids: ["b"] }] } as unknown as Snapshot;
  it("carries argument and return payload direction", () => {
    expect(annotatePayloads(snapshot, "a", "b")).toEqual({ argumentPayloads: ["Request"], returnPayloads: [] });
    expect(annotatePayloads(snapshot, "b", "a")).toEqual({ argumentPayloads: [], returnPayloads: ["Request"] });
  });
});

describe("flow presentation", () => {
  it("contains no execution order, steps, or animation flags", () => {
    const flow: FlowGraph = {
      horizon: 3, rootIds: ["a" as never],
      nodes: [{ id: "a" as never, label: "A", depth: 0, branch: false, truncationCount: 0, unknownContinuations: [] }],
      edges: [],
    };
    const serialized = JSON.stringify(flowPresentation(flow, "root" as never));
    expect(serialized).not.toMatch(/"(?:order|step|animated|animation)"/);
    expect(serialized).toContain('"presentation":"flow"');
  });
});
