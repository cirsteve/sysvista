import { useState } from "react";
import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ScopeCanvas } from "../components/organisms/ScopeCanvas";
import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";
import { LividScopeRenderer } from "../lib/livid/adapter";
import type { DiagramSpec, RenderedDiagram } from "../lib/livid/types";

const spec: DiagramSpec = {
  id: "scope:root", scopeId: "root" as DiagramSpec["scopeId"], semanticsProfile: "dependency",
  nodes: [
    { id: "directory", presentation: "directory", label: "src", deferredChildKey: "scope:src", details: { name: "src" } },
    { id: "file", presentation: "file", label: "file.ts", deferredChildKey: "scope:file", details: { path: "file.ts", language: "typescript", analysis: { kind: "none" } } },
    { id: "leaf", presentation: "symbol", label: "leaf", details: { name: "leaf", qualifiedName: "leaf", declarationKind: "function",
      fileId: "file" as never, span: { file_id: "file" as never, start_line: 1, start_column: 1, end_line: 1, end_column: 1 } } },
  ], edges: [],
};

function Harness({ diagram }: { diagram: RenderedDiagram | null }) {
  const [selected, setSelected] = useState<string | null>(null);
  const [location, setLocation] = useState("root");
  useKeyboardNavigation({ spec, selectedId: selected, onSelect: setSelected,
    onDescend: setLocation, onBack: () => setLocation("back") });
  return <>
    <output data-testid="selected">{selected}</output><output data-testid="location">{location}</output>
    <ScopeCanvas spec={spec} diagram={diagram} selectedId={selected} viewport={{ x: 0, y: 0, zoom: 1 }}
      onSelect={setSelected} onDescend={setLocation} onViewportChange={() => {}} />
  </>;
}

describe.each(["real", "fallback"] as const)("%s canvas keyboard navigation", (mode) => {
  it("selects, descends and handles Backspace", async () => {
    const user = userEvent.setup();
    const diagram = mode === "real" ? (await new LividScopeRenderer().renderSpec(spec)).diagram : null;
    render(<Harness diagram={diagram} />);
    await user.keyboard("{ArrowRight}");
    expect(screen.getByTestId("selected").textContent).toBe("file");
    await user.keyboard("{Enter}");
    expect(screen.getByTestId("location").textContent).toBe("scope:file");
    await user.keyboard("{Backspace}");
    expect(screen.getByTestId("location").textContent).toBe("back");
  });
  it("descends from a pointer action", async () => {
    const user = userEvent.setup();
    const diagram = mode === "real" ? (await new LividScopeRenderer().renderSpec(spec)).diagram : null;
    render(<Harness diagram={diagram} />);
    if (mode === "real") fireEvent.click(screen.getByRole("button", { name: "Descend into src" }));
    else await user.dblClick(screen.getByRole("button", { name: /src/i }));
    expect(screen.getByTestId("location").textContent).toBe("scope:src");
  });
});
