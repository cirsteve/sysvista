import ts from "typescript";
import type { Relationship } from "./contract.js";
import { type Declarations, moduleKey, symbolKey } from "./declarations.js";
import { relativePath, spanOf } from "./spans.js";

/**
 * `imports` edges from each owned module: default, named and namespace imports, plus
 * re-exports (`export { a } from`, `export * from`, `export * as ns from`). Named
 * bindings target the final declaration; namespace forms target the module.
 */
export function extractImports(program: ts.Program, root: string, declarations: Declarations, ownedFiles: Set<string>): Relationship[] {
  const checker = program.getTypeChecker(); const result: Relationship[] = [];
  for (const source of program.getSourceFiles()) {
    if (!ownedFiles.has(ts.sys.resolvePath(source.fileName))) continue;
    const sourceId = moduleKey(relativePath(root, source.fileName));
    const edge = (location: ts.Node, symbol: ts.Symbol | undefined, name: string) => {
      const target = symbolKey(checker, symbol, declarations);
      if (target) result.push({ kind: "imports", source: sourceId, target, origin: "resolved", name, span: spanOf(root, location) });
    };
    const moduleSymbol = (specifier: ts.Expression) => checker.getSymbolAtLocation(specifier);
    for (const statement of source.statements) {
      if (ts.isImportDeclaration(statement) && statement.importClause) {
        const clause = statement.importClause;
        if (clause.name) edge(clause.name, checker.getSymbolAtLocation(clause.name), clause.name.text);
        const bindings = clause.namedBindings;
        if (bindings && ts.isNamedImports(bindings)) for (const element of bindings.elements) edge(element.name, checker.getSymbolAtLocation(element.name), element.name.text);
        if (bindings && ts.isNamespaceImport(bindings)) edge(bindings.name, moduleSymbol(statement.moduleSpecifier), `* as ${bindings.name.text}`);
      } else if (ts.isExportDeclaration(statement) && statement.moduleSpecifier) {
        const clause = statement.exportClause;
        if (clause && ts.isNamedExports(clause)) for (const element of clause.elements) edge(element.name, checker.getSymbolAtLocation(element.name), element.name.text);
        else edge(statement, moduleSymbol(statement.moduleSpecifier), clause ? `* as ${clause.name.getText()}` : "*");
      }
    }
  }
  return result;
}
