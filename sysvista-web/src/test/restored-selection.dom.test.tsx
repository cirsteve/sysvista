import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { ScopeCanvas } from "../components/organisms/ScopeCanvas";
import { LividScopeRenderer } from "../lib/livid/adapter";
import { fixtureProjection } from "../lib/livid/fake";
import { encodeViewState, decodeViewState } from "../lib/view-state/hash";
import { defaultViewState } from "../lib/view-state/types";

describe.each(["real", "fallback"] as const)("%s restored selection", (mode) => {
  it.each(["node", "edge"] as const)("restores a %s selection from the URL", async (kind) => {
    const base = new LividScopeRenderer().toDiagramSpec(fixtureProjection());
    const spec = { ...base, edges: [{ id: "edge", presentation: "aggregate-edge" as const,
      source: base.nodes[0].id, target: base.nodes[1].id, label: "calls",
      details: { kind: "calls", origin: "resolved", count: 1, relationshipIds: ["underlying"] } }] };
    const selected = kind === "node" ? spec.nodes[0].id : spec.edges[0].id;
    const restored = decodeViewState(encodeViewState({ ...defaultViewState(), scopeId: spec.scopeId, selection: selected })).state;
    const diagram = mode === "real" ? (await new LividScopeRenderer().renderSpec(spec)).diagram : null;
    render(<ScopeCanvas spec={spec} diagram={diagram} selectedId={restored.selection}
      viewport={restored.viewport} onSelect={() => {}} onDescend={() => {}} onViewportChange={() => {}} />);
    if (mode === "fallback") expect(screen.getAllByRole("button").some((button) => button.getAttribute("aria-pressed") === "true")).toBe(true);
    else expect(screen.getByRole("application", { name: `Dependency diagram for ${spec.scopeId}` })).toBeTruthy();
  });
});
