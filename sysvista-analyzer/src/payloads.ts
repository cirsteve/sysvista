import ts from "typescript";
import type { Payload } from "./contract.js";
import { type Declarations } from "./declarations.js";

export function extractPayloads(program: ts.Program, declarations: Declarations, ownedFiles: Set<string>): Payload[] {
  const checker = program.getTypeChecker(); const byName = new Map<string, Payload>();
  for (const [node, key] of declarations.nodes) {
    if (!ts.isFunctionLike(node) || !ownedFiles.has(ts.sys.resolvePath(node.getSourceFile().fileName))) continue;
    const signature = checker.getSignatureFromDeclaration(node); if (!signature) continue;
    for (const parameter of node.parameters) { const name = checker.typeToString(checker.getTypeAtLocation(parameter)); if (name && name !== "any" && name !== "unknown") { const payload = byName.get(name) ?? { name, producers: [], consumers: [] }; payload.consumers.push(key); byName.set(name, payload); } }
    const name = checker.typeToString(checker.getReturnTypeOfSignature(signature)); if (name && !["void", "any", "unknown"].includes(name)) { const payload = byName.get(name) ?? { name, producers: [], consumers: [] }; payload.producers.push(key); byName.set(name, payload); }
  }
  return [...byName.values()];
}
