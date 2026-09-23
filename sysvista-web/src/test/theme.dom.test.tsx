import { beforeEach, describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ThemeToggle } from "../components/atoms/ThemeToggle";
import { ScopeCanvas } from "../components/organisms/ScopeCanvas";
import type { RenderedDiagram } from "../lib/livid/types";
import { useViewStore } from "../store/viewStore";
import { acceptanceSpec, canvasForMode } from "./setup.dom";

function ThemeHarness({ diagram }: { diagram: RenderedDiagram | null }) {
  const theme = useViewStore((state) => state.theme);
  const toggle = useViewStore((state) => state.toggleTheme);
  return <div data-testid="shell" className={theme === "dark" ? "dark" : ""}>
    <ThemeToggle theme={theme} onToggle={toggle} />
    <ScopeCanvas spec={acceptanceSpec} diagram={diagram} selectedId={null} viewport={{ x: 0, y: 0, zoom: 1 }}
      onSelect={() => {}} onDescend={() => {}} onViewportChange={() => {}} />
  </div>;
}
beforeEach(() => useViewStore.setState({ theme: "light" }));

describe.each(["real", "fallback"] as const)("%s theme", (mode) => {
  it("toggles the dark ancestor used by Tailwind v4", async () => {
    const user = userEvent.setup();
    render(<ThemeHarness diagram={await canvasForMode(mode)} />);
    expect(mode === "real" ? screen.getByRole("application", { name: "Dependency diagram for root" }) : screen.getByRole("button", { name: /src/i })).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Use dark theme" }));
    expect(screen.getByTestId("shell").classList.contains("dark")).toBe(true);
  });
});
