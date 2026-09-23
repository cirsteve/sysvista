import { describe, expect, it } from "vitest";
import { strToU8, zipSync } from "fflate";
import sample from "../test/fixtures/v1/sample-output.json";
import { loadFromArchive, loadFromFile, loadFromFiles, validate } from "./loader";

const manifest = { schema_version: "3", root_scope_id: "root", repository: "example/repo", scanned_at: "2026-09-21T00:00:00Z", root: "/repo", tool_version: "0.1.0", inventory: { included: 1, excluded: 0, unsupported: 0, unreadable: 0, failed: 0 } };
const emptyIndex = { scopes: [{ scope_id: "root", children: [], owner_map: {}, crossing_relationship_ids: [] }] };
const jsonFile = (name: string, value: unknown, relativePath = name) => ({
  name,
  webkitRelativePath: relativePath,
  text: async () => JSON.stringify(value),
}) as File;

describe("loader validate", () => {
  it("rejects schema version 2 with the expected version", () => {
    const result = validate({ manifest: { ...manifest, schema_version: "2" }, scope_index: emptyIndex });
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.error.message).toContain("3");
  });
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
    const result = validate({ manifest, graph: { entities: [], relationships: [] }, scope_index: emptyIndex, diagnostics: [] });
    expect(result.ok && result.value.origin).toBe("v2");
  });
  it("loads the metadata files emitted by the v2 CLI", async () => {
    const result = await loadFromFiles([
      jsonFile("manifest.json", manifest),
      jsonFile("graph.json", { entities: [], relationships: [] }),
      jsonFile("diagnostics.json", []),
      jsonFile("findings.json", []),
      jsonFile("scopes.json", emptyIndex, "bundle/index/scopes.json"),
    ]);
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.origin).toBe("v2");
      expect(result.value.snapshot.scope_index).toEqual(emptyIndex);
    }
  });
  it("returns a load error when reading a file rejects", async () => {
    const file = { name: "broken.json", text: async () => { throw new Error("read failed"); } } as unknown as File;
    await expect(loadFromFile(file)).resolves.toEqual({ ok: false, error: { kind: "parse", message: "read failed" } });
  });
  it("rejects an oversized archive before reading its bytes", async () => {
    const file = {
      name: "huge.zip",
      size: 512 * 1024 * 1024 + 1,
      arrayBuffer: () => { throw new Error("must not read"); },
    } as unknown as File;
    const result = await loadFromFile(file);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.error).toMatchObject({ kind: "format", message: expect.stringContaining("cap") });
  });
  it("rejects an oversized folder member before reading it", async () => {
    const large = { name: "graph.json", webkitRelativePath: "bundle/graph.json", size: 512 * 1024 * 1024 + 1,
      text: () => { throw new Error("must not read"); } } as unknown as File;
    const result = await loadFromFiles([jsonFile("manifest.json", manifest), large,
      jsonFile("diagnostics.json", []), jsonFile("scopes.json", emptyIndex, "bundle/index/scopes.json")]);
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.error.message).toContain("cap");
  });
  it("reports the relationship ID for a dangling v2 target", () => {
    const result = validate({ manifest, scope_index: emptyIndex, source_files: [{ id: "f", path: "a.ts", analysis: { kind: "none" } }], entities: [{ id: "a", name: "a", qualified_name: "a", declaration_kind: "service", file_id: "f", scope_id: "s", span: { file_id: "f", start_line: 1, start_column: 1, end_line: 1, end_column: 1 } }], relationships: [{ id: "rel-dangling", kind: "calls", source: "a", target: "missing", origin: "test" }] });
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error.kind).toBe("references");
      expect(JSON.stringify(result.error)).toContain("rel-dangling");
    }
  });
  it("loads archive metadata and lazily resolves content-addressed source", async () => {
    const hash = "2689367b205c16ce32ed4200942b8b8b1e262dfc70d9bc9fbc77c49699a4f1df";
    const archive = zipSync({
      "manifest.json": strToU8(JSON.stringify({ ...manifest, source_included: true })),
      "graph.json": strToU8(JSON.stringify({ entities: [], relationships: [] })),
      "diagnostics.json": strToU8("[]"),
      "findings.json": strToU8("[]"),
      "index/scopes.json": strToU8(JSON.stringify(emptyIndex)),
      "source-index.json": strToU8(JSON.stringify({ files: [{ file_id: "f", path: "a.ts", content_hash: hash, byte_length: 2, source_available: true }] })),
      [`source/${hash}`]: strToU8("ok"),
    });
    const result = await loadFromArchive(archive);
    expect(result.ok).toBe(true);
    if (result.ok) {
      const first = await result.value.sources?.read(hash);
      expect(new TextDecoder().decode(first)).toBe("ok");
      expect(await result.value.sources?.read(hash)).toBe(first);
    }
  });
  it("uses findings embedded in graph metadata when the separate entry is absent", async () => {
    const embedded = [{ kind: "project", message: "embedded finding" }];
    const archive = zipSync({
      "manifest.json": strToU8(JSON.stringify(manifest)),
      "graph.json": strToU8(JSON.stringify({ entities: [], relationships: [], findings: embedded })),
      "diagnostics.json": strToU8("[]"),
      "index/scopes.json": strToU8(JSON.stringify(emptyIndex)),
    });
    const archived = await loadFromArchive(archive);
    expect(archived.ok && archived.value.snapshot.findings).toEqual(embedded);

    const directory = await loadFromFiles([
      jsonFile("manifest.json", manifest),
      jsonFile("graph.json", { entities: [], relationships: [], findings: embedded }),
      jsonFile("diagnostics.json", []),
      jsonFile("scopes.json", emptyIndex, "bundle/index/scopes.json"),
    ]);
    expect(directory.ok && directory.value.snapshot.findings).toEqual(embedded);
  });
  it("marks sources unavailable when a no-source archive is loaded", async () => {
    const archive = zipSync({
      "manifest.json": strToU8(JSON.stringify({ ...manifest, source_included: false })),
      "graph.json": strToU8(JSON.stringify({ entities: [], relationships: [] })),
      "diagnostics.json": strToU8("[]"),
      "findings.json": strToU8("[]"),
      "index/scopes.json": strToU8(JSON.stringify(emptyIndex)),
      "source-index.json": strToU8(JSON.stringify({ files: [{ file_id: "f", path: "a.ts", source_available: false }] })),
    });
    const result = await loadFromArchive(archive);
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.sources?.included).toBe(false);
      expect([...result.value.sources!.index.values()]).toEqual([
        expect.objectContaining({ file_id: "f", source_available: false }),
      ]);
    }
  });
});
