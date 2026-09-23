import { expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { LividDiagram } from "@rankonelabs/livid-react";
import { LividScopeRenderer } from "../lib/livid/adapter";
import { fixtureProjection } from "../lib/livid/fake";

it("mounts a real Livid diagram in jsdom", async () => {
  const result = await new LividScopeRenderer().render(fixtureProjection());
  expect(result.diagram).not.toBeNull();
  render(<LividDiagram diagram={result.diagram as never}
    frame={{ __brand: "StateFrame", nodes: {}, edges: {} }}
    ariaLabel="Livid mount spike" interactive />);
  expect(screen.getByRole("application", { name: "Livid mount spike" })).toBeTruthy();
});
