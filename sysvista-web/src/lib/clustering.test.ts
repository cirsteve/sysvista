import { describe, it, expect } from "vitest";
import { classifyComponents, detectHubs } from "./clustering";
import type { DetectedComponent, DetectedEdge } from "../types/schema";

function makeComponent(
  id: string,
  name: string,
  kind: DetectedComponent["kind"],
  file = "src/test.py",
  overrides: Partial<DetectedComponent> = {},
): DetectedComponent {
  return {
    id,
    name,
    kind,
    language: "python",
    source: { file },
    metadata: {},
    ...overrides,
  };
}

describe("classifyComponents", () => {
  it("groups models by CamelCase prefix", () => {
    const comps = [
      makeComponent("1", "SessionConfig", "model"),
      makeComponent("2", "SessionPeer", "model"),
      makeComponent("3", "SessionToken", "model"),
    ];
    const clusters = classifyComponents(comps);
    expect(clusters.get("1")).toBe("Session");
    expect(clusters.get("2")).toBe("Session");
    expect(clusters.get("3")).toBe("Session");
  });

  it("groups services by CamelCase prefix", () => {
    const comps = [
      makeComponent("1", "CommentGenerator", "service"),
      makeComponent("2", "CommentCritic", "service"),
      makeComponent("3", "CommentFormatter", "service"),
    ];
    const clusters = classifyComponents(comps);
    expect(clusters.get("1")).toBe("Comment");
    expect(clusters.get("2")).toBe("Comment");
    expect(clusters.get("3")).toBe("Comment");
  });

  it("groups transports by first HTTP path segment", () => {
    const comps = [
      makeComponent("1", "GET /sessions", "transport", "routes.py", {
        http_path: "/sessions",
      }),
      makeComponent("2", "POST /sessions/{id}", "transport", "routes.py", {
        http_path: "/sessions/{id}",
      }),
      makeComponent("3", "DELETE /sessions/{id}", "transport", "routes.py", {
        http_path: "/sessions/{id}",
      }),
    ];
    const clusters = classifyComponents(comps);
    expect(clusters.get("1")).toBe("Sessions");
    expect(clusters.get("2")).toBe("Sessions");
    expect(clusters.get("3")).toBe("Sessions");
  });

  it("falls back to source directory name", () => {
    const comps = [
      makeComponent("1", "x", "model", "src/utils/a.py"),
      makeComponent("2", "y", "model", "src/utils/b.py"),
      makeComponent("3", "z", "model", "src/utils/c.py"),
    ];
    const clusters = classifyComponents(comps);
    // lowercase names don't match CamelCase prefix, so fallback to dir
    expect(clusters.get("1")).toBe("utils");
    expect(clusters.get("2")).toBe("utils");
    expect(clusters.get("3")).toBe("utils");
  });

  it("folds clusters with fewer than 3 members into Other", () => {
    const comps = [
      makeComponent("1", "SessionConfig", "model"),
      makeComponent("2", "SessionPeer", "model"),
      makeComponent("3", "SessionToken", "model"),
      makeComponent("4", "LonelyModel", "model"), // "Lonely" cluster has 1 member
    ];
    const clusters = classifyComponents(comps);
    expect(clusters.get("1")).toBe("Session");
    expect(clusters.get("4")).toBe("Other");
  });

  it("assigns Other when no classification is possible", () => {
    const comps = [makeComponent("1", "x", "model", "root.py")];
    const clusters = classifyComponents(comps);
    // single-char lowercase name, file has no parent dir → "Other"
    expect(clusters.get("1")).toBe("Other");
  });
});

describe("detectHubs", () => {
  it("returns empty map for no components", () => {
    const hubs = detectHubs([], []);
    expect(hubs.size).toBe(0);
  });

  it("marks all nodes normal when degrees are uniform", () => {
    const comps = [
      makeComponent("a", "A", "service"),
      makeComponent("b", "B", "service"),
      makeComponent("c", "C", "service"),
    ];
    const edges: DetectedEdge[] = [
      { from_id: "a", to_id: "b" },
      { from_id: "b", to_id: "c" },
      { from_id: "c", to_id: "a" },
    ];
    const hubs = detectHubs(comps, edges);
    for (const [, info] of hubs) {
      expect(info.tier).toBe("normal");
      expect(info.degree).toBe(2);
    }
  });

  it("detects a high-degree hub", () => {
    // One node connected to many others; the rest have degree 1
    const comps = [
      makeComponent("hub", "Hub", "service"),
      ...Array.from({ length: 8 }, (_, i) =>
        makeComponent(`n${i}`, `N${i}`, "model"),
      ),
    ];
    const edges: DetectedEdge[] = Array.from({ length: 8 }, (_, i) => ({
      from_id: "hub",
      to_id: `n${i}`,
    }));

    const hubs = detectHubs(comps, edges);
    expect(hubs.get("hub")!.tier).toBe("high");
    expect(hubs.get("hub")!.degree).toBe(8);
    // Leaf nodes should be normal
    expect(hubs.get("n0")!.tier).toBe("normal");
    expect(hubs.get("n0")!.degree).toBe(1);
  });

  it("tracks degree correctly for bidirectional edges", () => {
    const comps = [
      makeComponent("a", "A", "service"),
      makeComponent("b", "B", "model"),
    ];
    const edges: DetectedEdge[] = [
      { from_id: "a", to_id: "b" },
      { from_id: "b", to_id: "a" },
    ];
    const hubs = detectHubs(comps, edges);
    expect(hubs.get("a")!.degree).toBe(2);
    expect(hubs.get("b")!.degree).toBe(2);
  });

  it("includes zero-degree nodes", () => {
    const comps = [
      makeComponent("a", "A", "service"),
      makeComponent("b", "B", "model"),
    ];
    const hubs = detectHubs(comps, []);
    expect(hubs.get("a")!.degree).toBe(0);
    expect(hubs.get("b")!.degree).toBe(0);
    expect(hubs.get("a")!.tier).toBe("normal");
  });
});
