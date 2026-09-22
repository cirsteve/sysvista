import ts from "typescript";
import type { Entity } from "./contract.js";
import { relativePath, spanOf } from "./spans.js";

/**
 * Canonical keys for declarations in every scanned file visible to one program.
 * `entities` holds only the owned files' declarations; `nodes` also covers scanned
 * files owned by other programs, so cross-project targets resolve to the same keys.
 * Keys are built from file, ownership chain, kind and per-file ordinal, never from
 * `ts.Symbol` identity, which is private to a program.
 */
export interface Declarations { entities: Entity[]; nodes: Map<ts.Node, string>; scannedFiles: Set<string> }

function isFunctionInitializer(node: ts.Expression | undefined): boolean {
  return !!node && (ts.isArrowFunction(node) || ts.isFunctionExpression(node));
}
function named(node: ts.Node): node is ts.Declaration & { name: ts.DeclarationName } {
  return (ts.isFunctionDeclaration(node) || ts.isClassDeclaration(node) || ts.isInterfaceDeclaration(node) || ts.isTypeAliasDeclaration(node) || ts.isEnumDeclaration(node) || ts.isMethodDeclaration(node) || ts.isConstructorDeclaration(node) || ts.isVariableDeclaration(node) || ts.isGetAccessorDeclaration(node) || ts.isSetAccessorDeclaration(node) || (ts.isPropertyDeclaration(node) && isFunctionInitializer(node.initializer))) && !!node.name;
}
function nameOf(node: ts.Declaration & { name: ts.DeclarationName }): string { return ts.isIdentifier(node.name) || ts.isStringLiteral(node.name) || ts.isNumericLiteral(node.name) ? node.name.text : node.name.getText(); }
function kindOf(node: ts.Node): string {
  if (ts.isFunctionDeclaration(node)) return "function";
  if (ts.isClassDeclaration(node)) return "class";
  if (ts.isInterfaceDeclaration(node)) return "interface";
  if (ts.isTypeAliasDeclaration(node)) return "type_alias";
  if (ts.isEnumDeclaration(node)) return "enum";
  if (ts.isMethodDeclaration(node)) return "method";
  if (ts.isConstructorDeclaration(node)) return "constructor";
  if (ts.isGetAccessorDeclaration(node)) return "getter";
  if (ts.isSetAccessorDeclaration(node)) return "setter";
  if (ts.isPropertyDeclaration(node)) return "property";
  return "variable";
}

export function moduleKey(file: string): string { return `${file}#<module>#module#0`; }
export function isScanned(declarations: Declarations, source: ts.SourceFile): boolean {
  return declarations.scannedFiles.has(ts.sys.resolvePath(source.fileName)) && !source.fileName.includes("/node_modules/");
}

export function extractDeclarations(program: ts.Program, root: string, scannedFiles: Set<string>, ownedFiles: Set<string>): Declarations {
  const entities: Entity[] = []; const nodes = new Map<ts.Node, string>(); const declarations = { entities, nodes, scannedFiles };
  program.getTypeChecker(); // binding sets the parent pointers spans rely on
  for (const source of program.getSourceFiles()) {
    if (!isScanned(declarations, source)) continue;
    const emit = ownedFiles.has(ts.sys.resolvePath(source.fileName));
    const file = relativePath(root, source.fileName);
    nodes.set(source, moduleKey(file));
    if (emit) entities.push({ name: "<module>", ownership_chain: "<module>", declaration_kind: "module", file, discriminator: 0, start_line: 1, start_column: 1, end_line: source.getLineAndCharacterOfPosition(source.end).line + 1, end_column: 1, attributes: {} });
    const counts = new Map<string, number>();
    const owners: { name: string; key: string }[] = [];
    const visit = (node: ts.Node) => {
      const isNamed = named(node);
      if (isNamed) {
        const name = nameOf(node); const ownership_chain = [...owners.map(owner => owner.name), name].join("."); const declaration_kind = kindOf(node);
        const group = `${file}#${ownership_chain}#${declaration_kind}`; const discriminator = counts.get(group) ?? 0; counts.set(group, discriminator + 1); const key = `${group}#${discriminator}`;
        if (emit) {
          const span = spanOf(root, node);
          entities.push({ name, ownership_chain, declaration_kind, file, discriminator, owner_key: owners.at(-1)?.key, start_line: span.start_line, start_column: span.start_column, end_line: span.end_line, end_column: span.end_column, attributes: { implementation: "body" in node && !!node.body } });
        }
        nodes.set(node, key); owners.push({ name, key });
      }
      ts.forEachChild(node, visit);
      if (isNamed) owners.pop();
    }; visit(source);
  }
  return declarations;
}

/** Nearest enclosing declaration's key; top-level code has none, so module entries are never call sources. */
export function enclosingKey(node: ts.Node, declarations: Declarations): string | undefined { for (let current: ts.Node | undefined = node; current && !ts.isSourceFile(current); current = current.parent) { const key = declarations.nodes.get(current); if (key) return key; } return undefined; }
export function resolvedSymbol(checker: ts.TypeChecker, symbol: ts.Symbol | undefined): ts.Symbol | undefined { const seen = new Set<ts.Symbol>(); while (symbol && (symbol.flags & ts.SymbolFlags.Alias) && !seen.has(symbol)) { seen.add(symbol); symbol = checker.getAliasedSymbol(symbol); } return symbol; }
/** Overloaded functions resolve to their implementation, the one declaration with a body. */
function implementationFirst(found: readonly ts.Declaration[]): ts.Declaration[] {
  const hasBody = (declaration: ts.Declaration) => ts.isFunctionLike(declaration) && "body" in declaration && !!declaration.body;
  return [...found.filter(hasBody), ...found.filter(declaration => !hasBody(declaration))];
}
export function symbolKey(checker: ts.TypeChecker, symbol: ts.Symbol | undefined, declarations: Declarations): string | undefined { symbol = resolvedSymbol(checker, symbol); for (const declaration of implementationFirst(symbol?.declarations ?? [])) { const key = declarations.nodes.get(declaration); if (key) return key; } return undefined; }
/** True when every declaration of the symbol lives outside the scanned tree: default libraries, packages or out-of-root files. */
export function isExternal(checker: ts.TypeChecker, symbol: ts.Symbol | undefined, declarations: Declarations): boolean {
  const resolved = resolvedSymbol(checker, symbol); const found = resolved?.declarations ?? [];
  return found.length > 0 && found.every(declaration => !isScanned(declarations, declaration.getSourceFile()));
}
