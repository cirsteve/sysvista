import { describe, expect, it } from "vitest";
import { createFixtureRenderer, fixtureProjection } from "./fake";
import { LividScopeRenderer } from "./adapter";
import { toDiagramSpec } from "./spec";
import type { DiagramSpec } from "./types";
import type { Relationship, ScopeId, Snapshot } from "../../types/v2";
import { indexSnapshot } from "../projection/children";
import { projectScope } from "../projection/project";

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

  it("drives select, descend, focus, and replacement through the renderer contract", () => {
    const renderer = createFixtureRenderer();
    const events: string[] = [];
    renderer.subscribe((event) => events.push(event.type));
    const spec = renderer.toDiagramSpec(fixtureProjection());
    renderer.select("a");
    renderer.descend(spec.scopeId, "scope:a-scope");
    renderer.focus("a", { x: 1, y: 2, zoom: 1.5 });
    renderer.replace("snapshot", spec.scopeId, spec);
    expect(events).toEqual(["select", "descend", "focus", "replace"]);
  });

  it("round-trips the fixture through real Livid validation and layout", async () => {
    const result = await new LividScopeRenderer().render(fixtureProjection());
    expect(result.diagnostic).toBeUndefined();
    expect(result.diagram).not.toBeNull();
    expect(JSON.stringify(result.diagram)).toContain("scope:a-scope");
  });

  it("accepts fan-out and a cycle under dependency semantics", async () => {
    const snapshot = structuredClone(fixtureProjection().snapshot) as Snapshot;
    snapshot.relationships?.push({
      id: "r-cycle", kind: "calls", origin: "analyzer", source: "a2", target: "a1",
    } as unknown as Relationship);
    const index = indexSnapshot(snapshot);
    const scopeId = "a-scope" as ScopeId;
    const result = await new LividScopeRenderer().render({
      snapshot,
      index,
      projected: projectScope(snapshot, index, scopeId),
    });
    expect(result.diagnostic).toBeUndefined();
    expect(result.diagram).not.toBeNull();
  });

  it("returns a Diagnostic when Livid rejects a scope", async () => {
    class InvalidRenderer extends LividScopeRenderer {
      override toDiagramSpec(projection: Parameters<LividScopeRenderer["toDiagramSpec"]>[0]): DiagramSpec {
        const spec = super.toDiagramSpec(projection);
        return { ...spec, edges: spec.edges.map((edge, index) => index === 0 ? { ...edge, target: "missing" } : edge) };
      }
    }
    const result = await new InvalidRenderer().render(fixtureProjection());
    expect(result.diagram).toBeNull();
    expect(result.diagnostic).toMatchObject({ kind: "warning" });
    expect(result.diagnostic?.message).toContain("validation failed");
  });
});
