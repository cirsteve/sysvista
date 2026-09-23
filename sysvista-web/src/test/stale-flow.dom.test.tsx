import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { useGraphData } from "../hooks/useGraphData";
import { FakeScopeRenderer, fixtureProjection } from "../lib/livid/fake";
import { buildHierarchyIndex } from "../lib/hierarchy/index";
import type { DiagramSpec, ScopeRenderResult } from "../lib/livid/types";
import { defaultViewState } from "../lib/view-state/types";
import { useViewStore } from "../store/viewStore";

// Keep the hook on its initial renderer so each flow request can be completed by the test.
vi.mock("../lib/livid/adapter", () => ({ loadScopeRenderer: () => new Promise(() => {}) }));

const { snapshot } = fixtureProjection();
const loaded = { snapshot, hierarchy: buildHierarchyIndex(snapshot), origin: "v2" as const };
const rootEntityId = snapshot.entities![0].id;

function Harness() {
  const graph = useGraphData();
  return <>
    <button onClick={() => graph.load(loaded)}>Load</button>
    <button onClick={() => graph.select(rootEntityId)}>Select root</button>
    <button onClick={() => graph.setLens("flow")}>Flow</button>
    <button onClick={() => graph.setFlowHops(1)}>One hop</button>
    <output data-testid="rendered-flow">{(graph.slice?.diagram as { request?: string } | null)?.request ?? "pending"}</output>
  </>;
}

beforeEach(() => useViewStore.setState({ view: defaultViewState(), diagnostics: [] }));

it("discards a stale flow render completion after a newer hop request finishes", async () => {
  const pending: Array<{ spec: DiagramSpec; complete: (result: ScopeRenderResult) => void }> = [];
  const renderSpec = vi.spyOn(FakeScopeRenderer.prototype, "renderSpec").mockImplementation((spec) =>
    new Promise<ScopeRenderResult>((complete) => pending.push({ spec, complete })));
  try {
    render(<Harness />);
    fireEvent.click(screen.getByRole("button", { name: "Load" }));
    fireEvent.click(screen.getByRole("button", { name: "Select root" }));
    fireEvent.click(screen.getByRole("button", { name: "Flow" }));
    await waitFor(() => expect(pending).toHaveLength(1));
    fireEvent.click(screen.getByRole("button", { name: "One hop" }));
    await waitFor(() => expect(pending).toHaveLength(2));

    await act(async () => pending[1].complete({ spec: pending[1].spec, diagram: { request: "new" } }));
    expect(screen.getByTestId("rendered-flow").textContent).toBe("new");
    await act(async () => pending[0].complete({ spec: pending[0].spec, diagram: { request: "stale" } }));
    expect(screen.getByTestId("rendered-flow").textContent).toBe("new");
  } finally {
    renderSpec.mockRestore();
  }
});
