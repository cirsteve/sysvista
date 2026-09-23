#!/usr/bin/env node
import { readFileSync, statSync } from "node:fs";
import { join, isAbsolute, resolve } from "node:path";
import { createRequire } from "node:module";
import { createServer } from "../sysvista-web/node_modules/vite/dist/node/index.js";

const require = createRequire(import.meta.url);
const { unzipSync, strFromU8 } = require("../sysvista-web/node_modules/fflate") as typeof import("../sysvista-web/node_modules/fflate");

const target = process.argv[2];
if (!target) throw new Error("usage: node scripts/load-bundle.ts <bundle-directory-or-zip>");
const zipped = statSync(target).isFile();
const entries: Record<string, Uint8Array> | undefined = zipped ? unzipSync(new Uint8Array(readFileSync(target))) : undefined;
const read = (name: string) => {
  const entry = entries?.[name];
  if (zipped && !entry) throw new Error(`Missing ${name}`);
  return JSON.parse(zipped ? strFromU8(entry!) : readFileSync(join(target, name), "utf8"));
};
const manifest = read("manifest.json");
const graph = read("graph.json");
const diagnostics = read("diagnostics.json");
const index = read("index/scopes.json");
if (manifest.schema_version !== "3") throw new Error(`Expected schema version 3, got ${manifest.schema_version}`);
// Analyzer syntax issues in intentionally malformed test fixtures and D7 ambiguity
// notices do not invalidate the bundle's reference contract.
const expectedNotices = new Set(["payload_identity_conflict", "analyzer_issue"]);
const validationDiagnostics = diagnostics.filter((item: { kind: string }) => !expectedNotices.has(item.kind));
if (validationDiagnostics.length) throw new Error(`Validation diagnostics: ${JSON.stringify(validationDiagnostics.slice(0, 5))}`);
const ids = new Set<string>();
for (const item of [
  ...graph.source_files, ...graph.entities, ...graph.modules, ...graph.relationships,
  ...(graph.evidence ?? []), ...(graph.claims ?? []), ...(graph.projections ?? []),
  ...(graph.findings ?? []).filter((finding: { id?: string }) => finding.id),
]) {
  if (ids.has(item.id)) throw new Error(`Duplicate id ${item.id}`);
  ids.add(item.id);
}
const scopes = new Set(index.scopes.map((scope: { scope_id: string }) => scope.scope_id));
if (!scopes.has(manifest.root_scope_id)) throw new Error("Missing root scope");
const byScope = new Map(index.scopes.map((scope: { scope_id: string }) => [scope.scope_id, scope]));
for (const scope of index.scopes) {
  const visible = new Set(scope.children.map((child: { kind: string; file_id?: string; module_id?: string; entity_id?: string; scope_id?: string }) =>
    child.kind === "file" ? child.file_id : child.kind === "module" ? child.module_id : child.kind === "symbol" ? child.entity_id : child.scope_id));
  for (const child of scope.children) {
    if (child.scope_id && !scopes.has(child.scope_id)) throw new Error(`Missing child scope ${child.scope_id}`);
  }
  for (const target of Object.values(scope.owner_map ?? {})) {
    if (!visible.has(target)) throw new Error(`Owner target ${target} is not visible in ${scope.scope_id}`);
  }
}
const active = new Set<string>();
const visited = new Set<string>();
function walk(scopeId: string): void {
  if (active.has(scopeId)) throw new Error(`Containment cycle at ${scopeId}`);
  if (visited.has(scopeId)) return;
  active.add(scopeId);
  for (const child of byScope.get(scopeId).children) if (child.scope_id) walk(child.scope_id);
  active.delete(scopeId);
  visited.add(scopeId);
}
for (const scopeId of scopes) walk(scopeId);
for (const file of graph.source_files) {
  if (isAbsolute(file.path) || /^[A-Za-z]:[\\/]/.test(file.path)) throw new Error(`Absolute source path ${file.path}`);
}
for (const entry of manifest.inventory_entries ?? []) {
  if (isAbsolute(entry.path) || /^[A-Za-z]:[\\/]/.test(entry.path)) throw new Error(`Absolute inventory path ${entry.path}`);
}
const server = await createServer({ root: resolve("sysvista-web"), configFile: false, logLevel: "silent", optimizeDeps: { noDiscovery: true },
  server: { middlewareMode: true, hmr: false, watch: { ignored: ["**/*"] } }, appType: "custom" });
try {
  const [{ validateSnapshot }, { validateReferences }, { buildHierarchyIndex }, { projectScope }] = await Promise.all([
    server.ssrLoadModule("/src/lib/validate/v2.ts"), server.ssrLoadModule("/src/lib/validate/references.ts"),
    server.ssrLoadModule("/src/lib/hierarchy/index.ts"), server.ssrLoadModule("/src/lib/projection/project.ts"),
  ]);
  const snapshot = { ...graph, manifest, diagnostics, scope_index: index };
  const shape = validateSnapshot(snapshot);
  if (!shape.ok) throw new Error(`Viewer schema validation failed: ${JSON.stringify(shape.error.slice(0, 5))}`);
  const references = validateReferences(snapshot);
  if (!references.ok) throw new Error(`Viewer reference validation failed: ${JSON.stringify(references.error.slice(0, 5))}`);
  const hierarchy = buildHierarchyIndex(snapshot);
  projectScope(snapshot, index, hierarchy.rootScopeId);
} finally {
  await server.close();
}
console.log(JSON.stringify({ bundle: target, files: graph.source_files.length, entities: graph.entities.length, scopes: scopes.size }));
