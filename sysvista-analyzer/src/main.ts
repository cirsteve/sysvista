import { readFileSync } from "node:fs";
import ts from "typescript";
import { ANALYZER_VERSION, CONTRACT_VERSION, type AnalyzeRequest, type AnalyzeResponse } from "./contract.js";
import { handshake } from "./handshake.js";
import { createProjects } from "./program.js";
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
    const response: AnalyzeResponse = { contract_version: CONTRACT_VERSION, analyzer_version: ANALYZER_VERSION, entities: [], relationships: [], unresolved: [], diagnostics: [], payloads: [] };
    if (request.contract_version !== CONTRACT_VERSION) response.diagnostics.push({ message: `contract mismatch: ${request.contract_version}`, severity: "error" });
    else for (const project of createProjects(request.root, request.files, request.tsconfig)) {
      const declarations = extractDeclarations(project.program, request.root, project.ownedFiles);
      response.entities.push(...declarations.entities);
      response.relationships.push(...extractImports(project.program, declarations, project.ownedFiles));
      const calls = extractCalls(project.program, request.root, declarations, project.ownedFiles);
      response.relationships.push(...calls.relationships); response.unresolved.push(...calls.unresolved);
      response.payloads.push(...extractPayloads(project.program, declarations));
      for (const diagnostic of [...project.program.getSyntacticDiagnostics(), ...project.program.getOptionsDiagnostics()]) {
        response.diagnostics.push({ message: tsMessage(diagnostic.messageText), severity: diagnostic.category === 1 ? "error" : "warning", span: diagnostic.file && diagnostic.start !== undefined ? spanOf(request.root, tokenAt(diagnostic.file, diagnostic.start)) : undefined });
      }
    }
    response.entities.sort((a, b) => `${a.file}#${a.ownership_chain}#${a.declaration_kind}#${a.discriminator}`.localeCompare(`${b.file}#${b.ownership_chain}#${b.declaration_kind}#${b.discriminator}`));
    response.relationships = unique(response.relationships, value => JSON.stringify(value));
    response.unresolved = unique(response.unresolved, value => JSON.stringify(value));
    process.stdout.write(JSON.stringify(response));
  } catch (error) {
    process.stderr.write(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}

function unique<T>(values: T[], key: (value: T) => string): T[] { const seen = new Set<string>(); return values.filter(value => { const id = key(value); if (seen.has(id)) return false; seen.add(id); return true; }); }
function tsMessage(message: import("typescript").DiagnosticMessageChain | string): string { return typeof message === "string" ? message : [message.messageText, ...(message.next ?? []).map(tsMessage)].join(" "); }
function tokenAt(file: import("typescript").SourceFile, position: number): import("typescript").Node { let found: import("typescript").Node = file; const visit = (node: import("typescript").Node) => { if (node.pos <= position && position <= node.end) { found = node; node.forEachChild(visit); } }; file.forEachChild(visit); return found; }
