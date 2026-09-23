import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Inspector } from "../components/organisms/Inspector";
import { SearchBar } from "../components/organisms/SearchBar";
import { ScopeCanvas } from "../components/organisms/ScopeCanvas";
import { acceptanceSpec, canvasForMode } from "./setup.dom";

describe.each(["real", "fallback"] as const)("%s accessibility controls", (mode) => {
  it("exposes a combobox and resizes the Inspector with the keyboard", async () => {
    const user = userEvent.setup();
    render(<><SearchBar query="" results={[]} onSearch={() => {}} onSelect={() => {}} />
      <Inspector item={null} loaded={null} />
      <ScopeCanvas spec={acceptanceSpec} diagram={await canvasForMode(mode)} selectedId={null} viewport={{ x: 0, y: 0, zoom: 1 }}
        onSelect={() => {}} onDescend={() => {}} onViewportChange={() => {}} /></>);
    expect(mode === "real" ? screen.getByRole("application", { name: "Dependency diagram for root" }) : screen.getByRole("button", { name: /src/i })).toBeTruthy();
    expect(screen.getByRole("combobox", { name: "Search symbols" })).toBeTruthy();
    const separator = screen.getByRole("separator", { name: "Resize inspector" });
    expect(separator.getAttribute("tabindex")).toBe("0");
    separator.focus();
    await user.keyboard("{ArrowLeft}");
    expect(separator.getAttribute("aria-valuenow")).toBe("336");
  });
});
