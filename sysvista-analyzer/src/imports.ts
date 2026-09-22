import ts from "typescript";
import type { Relationship } from "./contract.js";
import { type Declarations, resolvedSymbol, symbolKey } from "./declarations.js";

export function extractImports(program: ts.Program, declarations: Declarations, ownedFiles: Set<string>): Relationship[] {
  const checker = program.getTypeChecker(); const result: Relationship[] = [];
  for (const source of program.getSourceFiles()) {
    if (!ownedFiles.has(ts.sys.resolvePath(source.fileName))) continue;
    const sourceKey = declarations.entities.find(entity => entity.file.replaceAll("\\", "/") === source.fileName.replaceAll("\\", "/") || source.fileName.replaceAll("\\", "/").endsWith(`/${entity.file}`));
    const sourceId = sourceKey ? `${sourceKey.file}#${sourceKey.ownership_chain}#${sourceKey.declaration_kind}#${sourceKey.discriminator}` : undefined;
    if (!sourceId) continue;
    for (const statement of source.statements) if (ts.isImportDeclaration(statement) && statement.importClause) {
      const bindings: ts.Node[] = []; if (statement.importClause.name) bindings.push(statement.importClause.name); const named = statement.importClause.namedBindings; if (named && ts.isNamedImports(named)) bindings.push(...named.elements);
      for (const binding of bindings) {
        const location = ts.isImportSpecifier(binding) ? binding.name : binding; const symbol = resolvedSymbol(checker, checker.getSymbolAtLocation(location)); const target = symbolKey(checker, symbol, declarations);
        if (target) result.push({ kind: "imports", source: sourceId, target, origin: "resolved", name: location.getText() });
      }
    }
  }
  return result;
}
