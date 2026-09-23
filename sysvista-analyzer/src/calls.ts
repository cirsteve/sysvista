import ts from "typescript";
import type { Relationship, UnresolvedReference } from "./contract.js";
import { enclosingKey, isExternal, isScanned, symbolKey, type Declarations } from "./declarations.js";
import { spanOf } from "./spans.js";

/**
 * `calls` edges from each owned declaration. Callees declared only outside the
 * scanned tree (default libraries, packages) are external and not reported as
 * unresolved; dynamic receivers and unmodelled local callees are.
 */
export function extractCalls(program: ts.Program, root: string, declarations: Declarations, ownedFiles: Set<string>): { relationships: Relationship[]; unresolved: UnresolvedReference[] } {
  const checker = program.getTypeChecker(); const relationships: Relationship[] = []; const unresolved: UnresolvedReference[] = [];
  for (const source of program.getSourceFiles()) {
    if (!ownedFiles.has(ts.sys.resolvePath(source.fileName))) continue;
    const visit = (node: ts.Node) => {
      // `import()` loads a module; it is not a call to a declaration.
      if (ts.isCallExpression(node) && node.expression.kind !== ts.SyntaxKind.ImportKeyword) {
        const sourceKey = enclosingKey(node, declarations); if (sourceKey) {
          const signature = checker.getResolvedSignature(node); const symbol = checker.getSymbolAtLocation(node.expression); const signatureNode = signature?.declaration; const signatureSymbol = signatureNode ? checker.getSymbolAtLocation("name" in signatureNode && signatureNode.name ? signatureNode.name : signatureNode) : undefined; const callee = signatureSymbol ?? symbol; const target = symbolKey(checker, callee, declarations); const name = node.expression.getText(); const span = spanOf(root, node);
          if (target) relationships.push({ kind: "calls", source: sourceKey, target, origin: "resolved", name, span });
          // A local binding whose signature is declared by a package (a React state setter) is external too.
          else if (!isExternal(checker, callee, declarations) && !(signatureNode && !isScanned(declarations, signatureNode.getSourceFile()))) {
            const type = checker.getTypeAtLocation(ts.isPropertyAccessExpression(node.expression) ? node.expression.expression : node.expression); const partial = !!(type.flags & (ts.TypeFlags.Any | ts.TypeFlags.Unknown));
            relationships.push({ kind: "calls", source: sourceKey, origin: "partial", name, span });
            unresolved.push({ source: sourceKey, name, span, reason: partial ? "dynamic or any-typed receiver" : unresolvedReason(callee) });
          }
        }
      }
      ts.forEachChild(node, visit);
    }; visit(source);
  }
  return { relationships, unresolved };
}

/** Local callees that are not entities: callbacks passed in, interface members, or other unmodelled declarations. */
function unresolvedReason(callee: ts.Symbol | undefined): string {
  const declaration = callee?.declarations?.[0];
  if (!declaration) return "callee has no declaration";
  if (ts.isParameter(declaration) || ts.isBindingElement(declaration) || ts.isPropertySignature(declaration) || ts.isMethodSignature(declaration)) return "callback or interface member";
  return "callee is not a modelled declaration";
}
