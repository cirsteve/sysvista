import { describe, expect, it } from "vitest";
import fixture from "../../test/fixtures/flow/bounded.json";
import type { EntityId, Snapshot } from "../../types/v2";
import { expandFlow } from "./expand";

const snapshot = fixture as unknown as Snapshot;
const root = { kind: "entity" as const, entityId: "a" as EntityId };

describe("expandFlow", () => {
  it("expands to N hops and reports the cut frontier", () => {
    const two = expandFlow(snapshot, root, 2);
    expect(two.nodes.map(({ id }) => id)).toEqual(["a", "b", "c", "d", "e"]);
    expect(two.nodes.filter(({ truncationCount }) => truncationCount).map(({ id, truncationCount }) => [id, truncationCount])).toEqual([["d", 1]]);
    const three = expandFlow(snapshot, root, 3);
    expect(three.nodes.map(({ id }) => id)).toEqual(["a", "b", "c", "d", "e"]);
    expect(three.nodes.find(({ id }) => id === "e")?.truncationCount).toBe(0);
    expect(three.edges.find(({ id }) => id === "ea")?.backEdge).toBe(true);
  });

  it("marks a recursive call as a back edge without duplicating its node", () => {
    const recursive = { ...snapshot, relationships: [{ id: "aa", kind: "calls", source: "a", target: "a", origin: "analyzer" }] } as unknown as Snapshot;
    const flow = expandFlow(recursive, root);
    expect(flow.nodes.map(({ id }) => id)).toEqual(["a"]);
    expect(flow.edges).toEqual([expect.objectContaining({ id: "aa", backEdge: true })]);
  });

  it("shows partial and heuristic calls only as unknown continuations", () => {
    const uncertain = { ...snapshot, relationships: [
      { id: "partial", kind: "calls", source: "a", target: "b", origin: "partial" },
      { id: "guess", kind: "calls", source: "a", target: "c", origin: "heuristic" },
    ] } as unknown as Snapshot;
    const flow = expandFlow(uncertain, root);
    expect(flow.edges).toHaveLength(0);
    expect(flow.nodes[0].unknownContinuations.map(({ reason }) => reason)).toEqual(["partial", "heuristic"]);
  });
});
