import ts from "typescript";
import type { Relationship, UnresolvedReference } from "./contract.js";
import { enclosingKey, symbolKey, type Declarations } from "./declarations.js";
import { spanOf } from "./spans.js";

export function extractCalls(program: ts.Program, root: string, declarations: Declarations, ownedFiles: Set<string>): { relationships: Relationship[]; unresolved: UnresolvedReference[] } {
  const checker = program.getTypeChecker(); const relationships: Relationship[] = []; const unresolved: UnresolvedReference[] = [];
  for (const source of program.getSourceFiles()) {
    if (!ownedFiles.has(ts.sys.resolvePath(source.fileName))) continue;
    const visit = (node: ts.Node) => {
      if (ts.isCallExpression(node)) {
        const sourceKey = enclosingKey(node, declarations); if (sourceKey) {
          const signature = checker.getResolvedSignature(node); const symbol = checker.getSymbolAtLocation(node.expression); const signatureNode = signature?.declaration; const signatureSymbol = signatureNode ? checker.getSymbolAtLocation("name" in signatureNode && signatureNode.name ? signatureNode.name : signatureNode) : undefined; const target = symbolKey(checker, signatureSymbol ?? symbol, declarations); const name = node.expression.getText(); const span = spanOf(root, node);
          if (target) relationships.push({ kind: "calls", source: sourceKey, target, origin: "resolved", name, span });
          else { const type = checker.getTypeAtLocation(ts.isPropertyAccessExpression(node.expression) ? node.expression.expression : node.expression); const partial = !!(type.flags & (ts.TypeFlags.Any | ts.TypeFlags.Unknown)); relationships.push({ kind: "calls", source: sourceKey, origin: "partial", name, span }); unresolved.push({ source: sourceKey, name, span, reason: partial ? "dynamic or any-typed receiver" : "callee has no owned declaration" }); }
        }
      }
      ts.forEachChild(node, visit);
    }; visit(source);
  }
  return { relationships, unresolved };
}
