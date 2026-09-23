import { describe, expect, it } from "vitest";
import { createFixtureRenderer, fixtureProjection } from "./fake";
import { LividScopeRenderer } from "./adapter";
import { toDiagramSpec } from "./spec";
import type { DiagramSpec } from "./types";
import type { ScopeId, Snapshot } from "../../types/v2";
import { projectScope } from "../projection/project";

describe("toDiagramSpec from CLI hierarchy", () => {
  it("renders root directory and logical modules as navigable nodes", () => {
    const spec = toDiagramSpec(fixtureProjection());
    expect(spec.nodes.some((node) => node.presentation === "directory" && node.deferredChildKey)).toBe(true);
    expect(spec.nodes.some((node) => node.presentation === "module" && node.deferredChildKey)).toBe(true);
    expect(spec.nodes).toHaveLength(fixtureProjection().projected.children.length);
  });
  it("shows files under directories and symbols under files", () => {
    const base = fixtureProjection();
    const directory = base.projected.children.find((item) => item.kind === "directory");
    expect(directory).toBeDefined();
    const directoryProjection = projectScope(base.snapshot, base.index, directory!.scopeId);
    const directorySpec = toDiagramSpec({ ...base, projected: directoryProjection });
    expect(directorySpec.nodes.some((node) => node.presentation === "file" && node.deferredChildKey)).toBe(true);
    const file = directoryProjection.children.find((item) => item.kind === "file");
    expect(file).toBeDefined();
    const fileProjection = projectScope(base.snapshot, base.index, file!.scopeId!);
    const fileSpec = toDiagramSpec({ ...base, projected: fileProjection });
    expect(fileSpec.nodes.some((node) => node.presentation === "symbol" && !node.deferredChildKey)).toBe(true);
  });
  it("drives renderer events", () => {
    const renderer = createFixtureRenderer();
    const events: string[] = [];
    renderer.subscribe((event) => events.push(event.type));
    const spec = renderer.toDiagramSpec(fixtureProjection());
    renderer.select(spec.nodes[0].id);
    renderer.descend(spec.scopeId, spec.nodes[0].deferredChildKey!);
    renderer.focus(spec.nodes[0].id, { x: 1, y: 2, zoom: 1.5 });
    renderer.replace("snapshot", spec.scopeId, spec);
    expect(events).toEqual(["select", "descend", "focus", "replace"]);
  });
  it("round-trips CLI output through real Livid validation and layout", async () => {
    const result = await new LividScopeRenderer().render(fixtureProjection());
    expect(result.diagnostic).toBeUndefined();
    expect(result.diagram).not.toBeNull();
  });
  it("returns an over-budget variant before layout", async () => {
    const projection = fixtureProjection();
    const projected = { ...projection.projected, children: Array.from({ length: 301 }, (_, index) => ({
      kind: "directory" as const, id: `dir-${index}`, scopeId: `scope-${index}`, name: `Directory ${index}`,
    })) } as unknown as typeof projection.projected;
    const spec = toDiagramSpec({ ...projection, projected });
    expect(spec.overBudget).toBe(true);
    expect(spec.items).toHaveLength(301);
    expect((await new LividScopeRenderer().renderSpec(spec)).diagram).toBeNull();
  });
  it("reports a validation diagnostic for a dangling diagram edge", async () => {
    const base = toDiagramSpec(fixtureProjection());
    const invalid: DiagramSpec = { ...base, edges: [{ id: "bad", presentation: "aggregate-edge",
      source: base.nodes[0].id, target: "missing", label: "calls",
      details: { kind: "calls", origin: "resolved", count: 1, relationshipIds: ["bad"] } }] };
    const result = await new LividScopeRenderer().renderSpec(invalid);
    expect(result.diagram).toBeNull();
    expect(result.diagnostic?.message).toContain("validation failed");
  });
  it("only emits symbol deferred keys for nested declarations", () => {
    const projection = fixtureProjection();
    const entity = projection.snapshot.entities!.find((item) => item.name !== "<module>")!;
    const nestedScope = "nested-scope" as ScopeId;
    const projected = { ...projection.projected, children: [{ kind: "symbol", id: entity.id, scopeId: nestedScope, name: entity.name, entity },
      { kind: "symbol", id: "leaf", name: "leaf", entity: { ...entity, id: "leaf" } }] } as unknown as typeof projection.projected;
    const spec = toDiagramSpec({ ...projection, projected, snapshot: projection.snapshot as Snapshot });
    expect(spec.nodes.find(({ id }) => id === entity.id)?.deferredChildKey).toBe(`scope:${nestedScope}`);
    expect(spec.nodes.find(({ id }) => id === "leaf")?.deferredChildKey).toBeUndefined();
  });
});
