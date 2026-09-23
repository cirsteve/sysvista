import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SearchBar } from "../components/organisms/SearchBar";
import { ScopeCanvas } from "../components/organisms/ScopeCanvas";
import type { EntitySearchHit } from "../lib/search";
import { acceptanceSpec, canvasForMode } from "./setup.dom";

const results = [
  { entityId: "first", name: "First", qualifiedName: "First", owningScopeId: "scope:first", kind: "function" },
  { entityId: "second", name: "Second", qualifiedName: "Second", owningScopeId: "scope:second", kind: "function" },
] as EntitySearchHit[];

describe.each(["real", "fallback"] as const)("%s search navigation", (mode) => {
  it("announces ranked options and opens the keyboard selected scope", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const diagram = await canvasForMode(mode);
    render(<><SearchBar query="s" results={results} onSearch={() => {}} onSelect={onSelect} />
      <ScopeCanvas spec={acceptanceSpec} diagram={diagram} selectedId={null} viewport={{ x: 0, y: 0, zoom: 1 }}
        onSelect={() => {}} onDescend={() => {}} onViewportChange={() => {}} /></>);
    expect(mode === "real" ? screen.getByRole("application", { name: "Dependency diagram for root" }) : screen.getByRole("button", { name: /src/i })).toBeTruthy();
    const input = screen.getByRole("combobox", { name: "Search symbols" });
    await user.click(input);
    expect(input.getAttribute("aria-expanded")).toBe("true");
    await user.keyboard("{ArrowDown}{Enter}");
    expect(onSelect).toHaveBeenCalledWith(results[1]);
  });
});
