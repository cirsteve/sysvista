import { readFileSync } from "node:fs";
import ts from "typescript";
import { ANALYZER_VERSION, CONTRACT_VERSION, type AnalyzeRequest, type AnalyzeResponse, type Entity, type Payload, type Relationship, type UnresolvedReference } from "./contract.js";
import { handshake } from "./handshake.js";
import { compare, createProjects, libraryDirectory } from "./program.js";
import { extractDeclarations } from "./declarations.js";
import { extractImports } from "./imports.js";
import { extractCalls } from "./calls.js";
import { extractPayloads } from "./payloads.js";
import { spanOf } from "./spans.js";

if (process.argv.includes("--handshake")) {
  process.stdout.write(JSON.stringify(handshake()));
} else {
  try {
    const request = JSON.parse(readFileSync(0, "utf8")) as AnalyzeRequest;
    process.stdout.write(JSON.stringify(analyze(request)));
  } catch (error) {
    process.stderr.write(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}

function analyze(request: AnalyzeRequest): AnalyzeResponse {
  const response: AnalyzeResponse = { contract_version: CONTRACT_VERSION, analyzer_version: ANALYZER_VERSION, entities: [], relationships: [], unresolved: [], diagnostics: [], payloads: [] };
  if (request.contract_version !== CONTRACT_VERSION) {
    response.diagnostics.push({ message: `contract mismatch: ${request.contract_version}`, severity: "error" });
    return response;
  }
  const { projects, diagnostics } = createProjects(request.root, request.files, request.tsconfig);
  response.diagnostics.push(...diagnostics);
  const payloads: Payload[] = [];
  for (const project of projects) {
    const declarations = extractDeclarations(project.program, request.root, project.scannedFiles, project.ownedFiles);
    response.entities.push(...declarations.entities);
    response.relationships.push(...extractImports(project.program, request.root, declarations, project.ownedFiles));
    const calls = extractCalls(project.program, request.root, declarations, project.ownedFiles);
    response.relationships.push(...calls.relationships); response.unresolved.push(...calls.unresolved);
    payloads.push(...extractPayloads(project.program, declarations, project.ownedFiles));
    const owned = project.program.getSourceFiles().filter(source => project.ownedFiles.has(ts.sys.resolvePath(source.fileName)));
    for (const diagnostic of [...owned.flatMap(source => project.program.getSyntacticDiagnostics(source)), ...project.program.getOptionsDiagnostics()]) {
      response.diagnostics.push({ message: tsMessage(diagnostic.messageText), severity: diagnostic.category === ts.DiagnosticCategory.Error ? "error" : "warning", span: diagnostic.file && diagnostic.start !== undefined ? spanOf(request.root, tokenAt(diagnostic.file, diagnostic.start)) : undefined });
    }
  }
  // A file included by overlapping tsconfigs is emitted once per project with identical keys.
  response.entities = unique(response.entities, entityKey).sort((a, b) => compareEntities(a, b));
  response.relationships = unique(response.relationships, value => JSON.stringify(value));
  response.unresolved = unique(response.unresolved, value => JSON.stringify(value));
  reconcileOverlap(response);
  response.payloads = mergePayloads(payloads);
  const scrub = scrubber(request.root);
  response.diagnostics = unique(response.diagnostics.map(diagnostic => ({ ...diagnostic, message: scrub(diagnostic.message) })), value => JSON.stringify(value));
  return response;
}

function entityKey(entity: Entity): string { return `${entity.file}#${entity.ownership_chain}#${entity.declaration_kind}#${entity.discriminator}`; }
function compareEntities(a: Entity, b: Entity): number {
  return compare(a.file, b.file) || compare(a.ownership_chain, b.ownership_chain) || compare(a.declaration_kind, b.declaration_kind) || a.discriminator - b.discriminator;
}

/**
 * When overlapping projects disagree about a call site, one program resolved it and
 * another did not. Keep the resolved edge and drop the other program's partial edge
 * and unresolved entry for the same site.
 */
function reconcileOverlap(response: AnalyzeResponse): void {
  const site = (source: string, name: string | undefined, span: Relationship["span"]) => JSON.stringify([source, name, span]);
  const resolved = new Set(response.relationships.filter(edge => edge.kind === "calls" && edge.origin === "resolved").map(edge => site(edge.source, edge.name, edge.span)));
  response.relationships = response.relationships.filter(edge => !(edge.kind === "calls" && edge.origin === "partial" && resolved.has(site(edge.source, edge.name, edge.span))));
  response.unresolved = response.unresolved.filter((item: UnresolvedReference) => !resolved.has(site(item.source, item.name, item.span)));
}

function mergePayloads(payloads: Payload[]): Payload[] {
  const byName = new Map<string, { producers: Set<string>; consumers: Set<string> }>();
  for (const payload of payloads) {
    const merged = byName.get(payload.name) ?? { producers: new Set<string>(), consumers: new Set<string>() };
    payload.producers.forEach(key => merged.producers.add(key)); payload.consumers.forEach(key => merged.consumers.add(key));
    byName.set(payload.name, merged);
  }
  return [...byName].sort(([a], [b]) => compare(a, b)).map(([name, { producers, consumers }]) => ({ name, producers: [...producers].sort(compare), consumers: [...consumers].sort(compare) }));
}

/**
 * Remove host paths from compiler messages: the scan root becomes relative, the
 * embedded library directory is named, and any other absolute path (a referenced
 * config outside the root) keeps only its file name. Both separator styles are
 * matched, since Windows messages use backslashes.
 */
function scrubber(root: string): (message: string) => string {
  // Kept local: the entry block above runs before module-level constants are initialised.
  // An absolute POSIX or Windows path that starts a word or a quotation; group 1 is its last segment.
  const absolutePath = /(?<=^|[\s'"`(])(?:[A-Za-z]:)?[\\/](?:[^\s'"`\\/]+[\\/])*([^\s'"`\\/]+)/g;
  const forms = (path: string) => { const posix = path.replaceAll("\\", "/").replace(/\/$/, ""); return [[posix, "/"], [posix.replaceAll("/", "\\"), "\\"]] as const; };
  const libraries = forms(libraryDirectory()); const rootPaths = forms(ts.sys.resolvePath(root));
  return message => {
    for (const [path, separator] of libraries) message = message.replaceAll(`${path}${separator}`, "<typescript-lib>/");
    for (const [path, separator] of rootPaths) message = message.replaceAll(`${path}${separator}`, "").replaceAll(path, ".");
    return message.replace(absolutePath, "<external>/$1");
  };
}

function unique<T>(values: T[], key: (value: T) => string): T[] { const seen = new Set<string>(); return values.filter(value => { const id = key(value); if (seen.has(id)) return false; seen.add(id); return true; }); }
function tsMessage(message: ts.DiagnosticMessageChain | string): string { return ts.flattenDiagnosticMessageText(message, " "); }
function tokenAt(file: ts.SourceFile, position: number): ts.Node { let found: ts.Node = file; const visit = (node: ts.Node) => { if (node.pos <= position && position <= node.end) { found = node; node.forEachChild(visit); } }; file.forEachChild(visit); return found; }
