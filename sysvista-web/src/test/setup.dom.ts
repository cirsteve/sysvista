import { afterEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import { LividScopeRenderer } from "../lib/livid/adapter";
import type { DiagramSpec } from "../lib/livid/types";

afterEach(() => cleanup());

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
vi.stubGlobal("ResizeObserver", ResizeObserverStub);
window.matchMedia ??= ((query: string) => ({
  matches: false, media: query, onchange: null, addListener() {}, removeListener() {},
  addEventListener() {}, removeEventListener() {}, dispatchEvent() { return false; },
})) as typeof window.matchMedia;
Element.prototype.scrollIntoView ??= () => {};

export const acceptanceSpec: DiagramSpec = { id: "acceptance", scopeId: "root" as never, semanticsProfile: "dependency",
  nodes: [{ id: "directory", presentation: "directory", label: "src", deferredChildKey: "scope:src", details: { name: "src" } }], edges: [] };

export async function canvasForMode(mode: "real" | "fallback") {
  return mode === "real" ? (await new LividScopeRenderer().renderSpec(acceptanceSpec)).diagram : null;
}
