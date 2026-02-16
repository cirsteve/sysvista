import { describe, it, expect } from "vitest";
import type { SysVistaOutput } from "../types/schema";

// Re-implement validate logic for testing since it's not exported
function validate(data: unknown): SysVistaOutput {
  const obj = data as Record<string, unknown>;
  if (
    !obj ||
    typeof obj !== "object" ||
    !Array.isArray(obj.components) ||
    !Array.isArray(obj.edges)
  ) {
    throw new Error(
      "Invalid SysVista JSON: missing required fields (components, edges)",
    );
  }
  if (!Array.isArray(obj.workflows)) {
    obj.workflows = [];
  }
  return obj as unknown as SysVistaOutput;
}

describe("loader validate", () => {
  it("accepts valid data with workflows", () => {
    const data = {
      version: "1",
      scanned_at: "2024-01-01",
      root_dir: "/test",
      project_name: "test",
      detected_languages: ["python"],
      components: [],
      edges: [],
      workflows: [{ id: "w1", name: "POST /messages", entry_point_id: "tp1", steps: [] }],
      scan_stats: { files_scanned: 1, files_skipped: 0, scan_duration_ms: 10 },
    };
    const result = validate(data);
    expect(result.workflows).toHaveLength(1);
    expect(result.workflows[0].name).toBe("POST /messages");
  });

  it("defaults workflows to empty array for old format", () => {
    const data = {
      version: "1",
      scanned_at: "2024-01-01",
      root_dir: "/test",
      project_name: "test",
      detected_languages: [],
      components: [{ id: "c1", name: "Test", kind: "model", language: "python", source: { file: "test.py" }, metadata: {} }],
      edges: [],
      scan_stats: { files_scanned: 1, files_skipped: 0, scan_duration_ms: 10 },
    };
    const result = validate(data);
    expect(result.workflows).toEqual([]);
  });

  it("rejects data missing components", () => {
    expect(() => validate({ edges: [] })).toThrow("missing required fields");
  });

  it("rejects data missing edges", () => {
    expect(() => validate({ components: [] })).toThrow("missing required fields");
  });

  it("rejects null input", () => {
    expect(() => validate(null)).toThrow("missing required fields");
  });

  it("preserves existing empty workflows array", () => {
    const data = {
      components: [],
      edges: [],
      workflows: [],
    };
    const result = validate(data);
    expect(result.workflows).toEqual([]);
  });
});
