#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { createServer } from "../sysvista-web/node_modules/vite/dist/node/index.js";

const directory = process.argv[2];
const reportPath = process.argv[3];
if (!directory || !reportPath) throw new Error("usage: node scripts/measure-viewer.ts <bundle-folder> <report.json>");
const webRoot = resolve("sysvista-web");
const server = await createServer({ root: webRoot, configFile: false, logLevel: "silent", optimizeDeps: { noDiscovery: true },
  server: { middlewareMode: true, hmr: false, watch: { ignored: ["**/*"] } }, appType: "custom" });
try {
  const [{ validateSnapshot }, { validateReferences }, { buildHierarchyIndex }, { projectScope }, { toDiagramSpec }, { LividScopeRenderer }] = await Promise.all([
    server.ssrLoadModule("/src/lib/validate/v2.ts"),
    server.ssrLoadModule("/src/lib/validate/references.ts"),
    server.ssrLoadModule("/src/lib/hierarchy/index.ts"),
    server.ssrLoadModule("/src/lib/projection/project.ts"),
    server.ssrLoadModule("/src/lib/livid/spec.ts"),
    server.ssrLoadModule("/src/lib/livid/adapter.ts"),
  ]);
  const start = performance.now();
  const read = (name: string) => JSON.parse(readFileSync(resolve(directory, name), "utf8"));
  const manifest = read("manifest.json");
  const graph = read("graph.json");
  const snapshot = { ...graph, manifest, diagnostics: read("diagnostics.json"), scope_index: read("index/scopes.json") };
  const index = buildHierarchyIndex(snapshot);
  const loadIndexMs = performance.now() - start;

  const validationStart = performance.now();
  const shape = validateSnapshot(snapshot);
  if (!shape.ok) throw new Error(JSON.stringify(shape.error.slice(0, 5)));
  const refs = validateReferences(snapshot);
  if (!refs.ok) throw new Error(JSON.stringify(refs.error.slice(0, 5)));
  const findingCount = (snapshot.findings ?? []).length;
  const validationFindingsMs = performance.now() - validationStart;

  const projectionStart = performance.now();
  const root = index.rootScopeId;
  const projected = projectScope(snapshot, snapshot.scope_index, root);
  index.nearestValidScope(root);
  const projectionNavigationMs = performance.now() - projectionStart;

  const layoutStart = performance.now();
  const spec = toDiagramSpec({ snapshot, index: snapshot.scope_index, projected });
  const rendered = await new LividScopeRenderer().renderSpec(spec);
  const boundedLayoutMs = performance.now() - layoutStart;
  const report = { files: snapshot.source_files?.length ?? 0, entities: snapshot.entities?.length ?? 0,
    relationships: snapshot.relationships?.length ?? 0, findingCount,
    timings_ms: { load_index: loadIndexMs, projection_navigation: projectionNavigationMs,
      validation_findings: validationFindingsMs, bounded_layout: boundedLayoutMs },
    overBudget: !!spec.overBudget, layoutDiagnostic: rendered.diagnostic?.message ?? null };
  writeFileSync(reportPath, JSON.stringify(report, null, 2) + "\n");
  console.log(JSON.stringify(report));
} finally {
  await server.close();
}
