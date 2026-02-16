import { describe, it, expect } from "vitest";
import { buildGraph } from "./graph-adapter";
import type { SysVistaOutput, ComponentKind } from "../types/schema";

function makeScan(overrides: Partial<SysVistaOutput> = {}): SysVistaOutput {
  return {
    version: "1",
    scanned_at: "2024-01-01",
    root_dir: "/test",
    project_name: "test",
    detected_languages: ["python"],
    components: [],
    edges: [],
    workflows: [],
    scan_stats: { files_scanned: 1, files_skipped: 0, scan_duration_ms: 10 },
    ...overrides,
  };
}

function makeComponent(id: string, name: string, kind: ComponentKind) {
  return {
    id,
    name,
    kind,
    language: "python",
    source: { file: "test.py" },
    metadata: {},
  };
}

describe("buildGraph edge styling", () => {
  const allKinds = new Set<ComponentKind>(["model", "service", "transport", "transform"]);

  it("styles calls edges green", () => {
    const data = makeScan({
      components: [
        makeComponent("tp1", "route", "transport"),
        makeComponent("svc1", "handler", "service"),
      ],
      edges: [{ from_id: "tp1", to_id: "svc1", label: "calls" }],
    });

    const { edges } = buildGraph(data, allKinds);
    expect(edges).toHaveLength(1);
    expect(edges[0].style?.stroke).toBe("#22c55e");
    expect(edges[0].animated).toBe(true);
  });

  it("styles dispatches edges amber", () => {
    const data = makeScan({
      components: [
        makeComponent("tp1", "route", "transport"),
        makeComponent("svc1", "worker", "service"),
      ],
      edges: [{ from_id: "tp1", to_id: "svc1", label: "dispatches" }],
    });

    const { edges } = buildGraph(data, allKinds);
    expect(edges).toHaveLength(1);
    expect(edges[0].style?.stroke).toBe("#f59e0b");
  });

  it("styles payload edges pink", () => {
    const data = makeScan({
      components: [
        makeComponent("tp1", "route", "transport"),
        makeComponent("m1", "Message", "model"),
      ],
      edges: [{ from_id: "tp1", to_id: "m1", label: "produces", payload_type: "Message" }],
    });

    const { edges } = buildGraph(data, allKinds);
    expect(edges).toHaveLength(1);
    expect(edges[0].style?.stroke).toBe("#f472b6");
  });

  it("styles regular flow edges cyan", () => {
    const data = makeScan({
      components: [
        makeComponent("svc1", "service", "service"),
        makeComponent("tp1", "route", "transport"),
      ],
      edges: [{ from_id: "svc1", to_id: "tp1", label: "handles" }],
    });

    const { edges } = buildGraph(data, allKinds);
    expect(edges).toHaveLength(1);
    expect(edges[0].style?.stroke).toBe("#06b6d4");
  });

  it("styles import edges gray", () => {
    const data = makeScan({
      components: [
        makeComponent("svc1", "service", "service"),
        makeComponent("m1", "Model", "model"),
      ],
      edges: [{ from_id: "svc1", to_id: "m1", label: "imports" }],
    });

    const { edges } = buildGraph(data, allKinds);
    expect(edges).toHaveLength(1);
    expect(edges[0].style?.stroke).toBe("#6b7280");
  });
});

describe("buildGraph filtering", () => {
  it("filters by active kinds", () => {
    const data = makeScan({
      components: [
        makeComponent("m1", "Model", "model"),
        makeComponent("tp1", "route", "transport"),
      ],
      edges: [{ from_id: "tp1", to_id: "m1", label: "persists" }],
    });

    const onlyModels = new Set<ComponentKind>(["model"]);
    const { nodes, edges } = buildGraph(data, onlyModels);
    // Only model node (no transport), so edge is filtered out
    expect(nodes).toHaveLength(1);
    expect(edges).toHaveLength(0);
  });

  it("deduplicates edges between same pair", () => {
    const data = makeScan({
      components: [
        makeComponent("a", "A", "service"),
        makeComponent("b", "B", "model"),
      ],
      edges: [
        { from_id: "a", to_id: "b", label: "imports" },
        { from_id: "a", to_id: "b", label: "references" },
      ],
    });

    const allKinds = new Set<ComponentKind>(["model", "service", "transport", "transform"]);
    const { edges } = buildGraph(data, allKinds);
    // Should be merged into one edge
    expect(edges).toHaveLength(1);
    expect(edges[0].label).toContain("imports");
    expect(edges[0].label).toContain("references");
  });
});
