import { describe, expect, it } from "vitest";
import sample from "../test/fixtures/v1/sample-output.json";
import { validate } from "./loader";

const manifest = { schema_version: "2", repository: "example/repo", scanned_at: "2026-09-21T00:00:00Z", root: "/repo", tool_version: "0.1.0", inventory: { included: 1, excluded: 0, unsupported: 0, unreadable: 0, failed: 0 } };

describe("loader validate", () => {
  it("adapts v1 and supplies one legacy diagnostic with unknown coverage", () => {
    const result = validate(sample);
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.origin).toBe("v1-legacy");
    expect(result.value.snapshot.diagnostics).toHaveLength(1);
    expect(result.value.snapshot.coverage).toBe("unknown");
    const persists = result.value.snapshot.relationships?.find(({ kind }) => kind === "persists");
    const nonPersists = result.value.snapshot.relationships?.find(({ kind }) => kind !== "persists");
    expect(persists).toMatchObject({ origin: "heuristic", rule: "model_name_match" });
    expect(nonPersists).toMatchObject({ origin: "heuristic" });
    expect(nonPersists).not.toHaveProperty("rule");
  });
  it("defaults omitted v1 workflows to empty", () => {
    const withoutWorkflows = { ...sample, workflows: undefined };
    const result = validate(withoutWorkflows);
    expect(result.ok && result.value.snapshot.claims).toEqual([]);
  });
  it("accepts a v2 bundle", () => {
    const result = validate({ manifest, graph: { entities: [], relationships: [] }, diagnostics: [] });
    expect(result.ok && result.value.origin).toBe("v2");
  });
  it("reports the relationship ID for a dangling v2 target", () => {
    const result = validate({ manifest, source_files: [{ id: "f", path: "a.ts", analysis: { kind: "none" } }], entities: [{ id: "a", name: "a", qualified_name: "a", declaration_kind: "service", file_id: "f", scope_id: "s", span: { file_id: "f", start_line: 1, start_column: 1, end_line: 1, end_column: 1 } }], relationships: [{ id: "rel-dangling", kind: "calls", source: "a", target: "missing", origin: "test" }] });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.kind).toBe("references");
      expect(JSON.stringify(result.error)).toContain("rel-dangling");
    }
  });
});
