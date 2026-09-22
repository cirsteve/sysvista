import { describe, expect, it } from "vitest";
import { strToU8, zipSync } from "fflate";
import sample from "../test/fixtures/v1/sample-output.json";
import { loadFromArchive, loadFromFile, loadFromFiles, validate } from "./loader";

const manifest = { schema_version: "2", repository: "example/repo", scanned_at: "2026-09-21T00:00:00Z", root: "/repo", tool_version: "0.1.0", inventory: { included: 1, excluded: 0, unsupported: 0, unreadable: 0, failed: 0 } };
const jsonFile = (name: string, value: unknown, relativePath = name) => ({
  name,
  webkitRelativePath: relativePath,
  text: async () => JSON.stringify(value),
}) as File;

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
  it("loads the metadata files emitted by the v2 CLI", async () => {
    const result = await loadFromFiles([
      jsonFile("manifest.json", manifest),
      jsonFile("graph.json", { entities: [], relationships: [] }),
      jsonFile("diagnostics.json", []),
      jsonFile("findings.json", []),
      jsonFile("scopes.json", { scopes: [] }, "bundle/index/scopes.json"),
    ]);
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.origin).toBe("v2");
      expect(result.value.snapshot.scope_index).toEqual({ scopes: [] });
    }
  });
  it("returns a load error when reading a file rejects", async () => {
    const file = { name: "broken.json", text: async () => { throw new Error("read failed"); } } as unknown as File;
    await expect(loadFromFile(file)).resolves.toEqual({ ok: false, error: { kind: "parse", message: "read failed" } });
  });
  it("reports the relationship ID for a dangling v2 target", () => {
    const result = validate({ manifest, source_files: [{ id: "f", path: "a.ts", analysis: { kind: "none" } }], entities: [{ id: "a", name: "a", qualified_name: "a", declaration_kind: "service", file_id: "f", scope_id: "s", span: { file_id: "f", start_line: 1, start_column: 1, end_line: 1, end_column: 1 } }], relationships: [{ id: "rel-dangling", kind: "calls", source: "a", target: "missing", origin: "test" }] });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.kind).toBe("references");
      expect(JSON.stringify(result.error)).toContain("rel-dangling");
    }
  });
  it("loads archive metadata and lazily resolves content-addressed source", async () => {
    const hash = "abc123";
    const archive = zipSync({
      "manifest.json": strToU8(JSON.stringify({ ...manifest, source_included: true })),
      "graph.json": strToU8(JSON.stringify({ entities: [], relationships: [] })),
      "diagnostics.json": strToU8("[]"),
      "findings.json": strToU8("[]"),
      "index/scopes.json": strToU8('{"scopes":[]}'),
      "source-index.json": strToU8(JSON.stringify({ files: [{ file_id: "f", path: "a.ts", content_hash: hash, byte_length: 2, source_available: true }] })),
      [`source/${hash}`]: strToU8("ok"),
    });
    const result = await loadFromArchive(archive);
    expect(result.ok).toBe(true);
    if (result.ok) expect(new TextDecoder().decode(await result.value.sources?.read(hash))).toBe("ok");
  });
});
