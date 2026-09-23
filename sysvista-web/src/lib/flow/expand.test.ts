import { describe, expect, it } from "vitest";
import cliFixture from "../../test/fixtures/flow/bounded.json";
import type { EntityId, Snapshot } from "../../types/v2";
import { expandFlow } from "./expand";

const ids = ["a", "b", "c", "d", "e"];
const snapshot = {
  entities: ids.map((id) => ({ id, name: id })),
  relationships: [
    ["ab", "a", "b"], ["ac", "a", "c"], ["bd", "b", "d"],
    ["ce", "c", "e"], ["de", "d", "e"], ["ea", "e", "a"],
  ].map(([id, source, target]) => ({ id, source, target, kind: "calls", origin: "analyzer" })),
} as unknown as Snapshot;
const root = { kind: "entity" as const, entityId: "a" as EntityId };

describe("expandFlow", () => {
  it("expands to N hops and reports the cut frontier", () => {
    const two = expandFlow(snapshot, root, 2);
    expect(two.nodes.map(({ id }) => id)).toEqual(ids);
    expect(two.nodes.filter(({ truncationCount }) => truncationCount).map(({ id, truncationCount }) => [id, truncationCount])).toEqual([["d", 1]]);
    const three = expandFlow(snapshot, root, 3);
    expect(three.nodes.find(({ id }) => id === "e")?.truncationCount).toBe(0);
    expect(three.edges.find(({ id }) => id === "ea")?.backEdge).toBe(true);
  });
  it("marks a recursive call as a back edge", () => {
    const recursive = { ...snapshot, relationships: [{ id: "aa", kind: "calls", source: "a", target: "a", origin: "analyzer" }] } as unknown as Snapshot;
    expect(expandFlow(recursive, root).edges).toEqual([expect.objectContaining({ id: "aa", backEdge: true })]);
  });
  it("marks a cycle across discovered branches", () => {
    const crossBranch = { ...snapshot, relationships: [
      { id: "ab", kind: "calls", source: "a", target: "b", origin: "analyzer" },
      { id: "ac", kind: "calls", source: "a", target: "c", origin: "analyzer" },
      { id: "bc", kind: "calls", source: "b", target: "c", origin: "analyzer" },
      { id: "cb", kind: "calls", source: "c", target: "b", origin: "analyzer" },
    ] } as unknown as Snapshot;
    const flow = expandFlow(crossBranch, root);
    expect(flow.edges.find(({ id }) => id === "cb")?.backEdge).toBe(true);
    expect(flow.edges.find(({ id }) => id === "bc")?.backEdge).toBe(false);
  });
  it("shows partial and heuristic calls as unknown continuations", () => {
    const uncertain = { ...snapshot, relationships: [
      { id: "partial", kind: "calls", source: "a", target: "b", origin: "partial" },
      { id: "guess", kind: "calls", source: "a", target: "c", origin: "heuristic" },
    ] } as unknown as Snapshot;
    const flow = expandFlow(uncertain, root);
    expect(flow.edges).toHaveLength(0);
    expect(flow.nodes[0].unknownContinuations.map(({ reason }) => reason)).toEqual(["partial", "heuristic"]);
  });
  it("accepts a CLI generated fixture", () => {
    const cli = cliFixture as unknown as Snapshot;
    const entity = cli.entities!.find(({ name }) => name !== "<module>")!;
    const flow = expandFlow(cli, { kind: "entity", entityId: entity.id }, 2);
    expect(flow.nodes[0]?.id).toBe(entity.id);
  });
});
