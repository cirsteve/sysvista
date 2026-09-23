import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useUrlViewState } from "../hooks/useUrlViewState";
import { useViewStore } from "../store/viewStore";
import { defaultViewState } from "../lib/view-state/types";
import { ScopeCanvas } from "../components/organisms/ScopeCanvas";
import { LividScopeRenderer } from "../lib/livid/adapter";
import type { DiagramSpec, RenderedDiagram } from "../lib/livid/types";

const spec: DiagramSpec = { id: "root", scopeId: "root" as never, semanticsProfile: "dependency",
  nodes: [{ id: "directory", presentation: "directory", label: "src", deferredChildKey: "child", details: { name: "src" } }], edges: [] };

function Harness() {
  const view = useUrlViewState();
  const navigate = useViewStore((state) => state.navigateScope);
  const update = useViewStore((state) => state.updateView);
  return <>
    <output data-testid="scope">{view.scopeId}</output>
    <button onClick={() => navigate("child" as never)}>Descend</button>
    <button onClick={() => update({ viewport: { x: 10, y: 20, zoom: 1.5 } })}>Pan</button>
  </>;
}

function CanvasHarness({ diagram }: { diagram: RenderedDiagram | null }) {
  const view = useUrlViewState();
  const navigate = useViewStore((state) => state.navigateScope);
  const update = useViewStore((state) => state.updateView);
  return <>
    <output data-testid="scope">{view.scopeId}</output>
    <button onClick={() => update({ viewport: { x: 10, y: 20, zoom: 1.5 } })}>Pan</button>
    <ScopeCanvas spec={spec} diagram={diagram} selectedId={view.selection} viewport={view.viewport}
      onSelect={(selection) => update({ selection })} onDescend={(scope) => navigate(scope as never)}
      onViewportChange={(viewport) => update({ viewport })} />
  </>;
}

beforeEach(() => {
  window.history.replaceState(null, "", "/");
  useViewStore.setState({ view: defaultViewState() });
});

describe("URL history", () => {
  it("pushes one entry per scope change and replaces viewport changes", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    const initial = window.history.length;
    await user.click(screen.getByRole("button", { name: "Descend" }));
    await waitFor(() => expect(window.location.hash).toContain("child"));
    expect(window.history.length).toBe(initial + 1);
    await user.click(screen.getByRole("button", { name: "Pan" }));
    await waitFor(() => expect(window.location.hash).toContain("10"));
    expect(window.history.length).toBe(initial + 1);
    window.history.back();
    await waitFor(() => expect(screen.getByTestId("scope").textContent).not.toBe("child"));
    window.history.forward();
    await waitFor(() => expect(screen.getByTestId("scope").textContent).toBe("child"));
    expect(window.history.length).toBe(initial + 1);
  });

  it.each(["real", "fallback"] as const)("restores the prior scope after pan and %s canvas descent", async (mode) => {
    const user = userEvent.setup();
    const diagram = mode === "real" ? (await new LividScopeRenderer().renderSpec(spec)).diagram : null;
    render(<CanvasHarness diagram={diagram} />);
    const initial = window.history.length;
    await user.click(screen.getByRole("button", { name: "Pan" }));
    await waitFor(() => expect(window.location.hash).toContain("10"));
    expect(window.history.length).toBe(initial);
    if (mode === "real") fireEvent.click(screen.getByRole("button", { name: "Descend into src" }));
    else await user.dblClick(screen.getByRole("button", { name: /src/i }));
    await waitFor(() => expect(screen.getByTestId("scope").textContent).toBe("child"));
    expect(window.history.length).toBe(initial + 1);
    window.history.back();
    await waitFor(() => expect(screen.getByTestId("scope").textContent).not.toBe("child"));
    window.history.forward();
    await waitFor(() => expect(screen.getByTestId("scope").textContent).toBe("child"));
  });
});
