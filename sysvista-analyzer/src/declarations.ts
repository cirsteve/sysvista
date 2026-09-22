import ts from "typescript";
import type { Entity } from "./contract.js";
import { relativePath, spanOf } from "./spans.js";

export interface Declarations { entities: Entity[]; nodes: Map<ts.Node, string>; symbols: Map<ts.Symbol, string> }

function named(node: ts.Node): node is ts.Declaration & { name: ts.DeclarationName } {
  return (ts.isFunctionDeclaration(node) || ts.isClassDeclaration(node) || ts.isInterfaceDeclaration(node) || ts.isTypeAliasDeclaration(node) || ts.isEnumDeclaration(node) || ts.isMethodDeclaration(node) || ts.isConstructorDeclaration(node) || ts.isVariableDeclaration(node)) && !!node.name;
}
function nameOf(node: ts.Declaration & { name: ts.DeclarationName }): string { return ts.isIdentifier(node.name) || ts.isStringLiteral(node.name) || ts.isNumericLiteral(node.name) ? node.name.text : node.name.getText(); }
function kindOf(node: ts.Node): string { if (ts.isFunctionDeclaration(node)) return "function"; if (ts.isClassDeclaration(node)) return "class"; if (ts.isInterfaceDeclaration(node)) return "interface"; if (ts.isTypeAliasDeclaration(node)) return "type_alias"; if (ts.isEnumDeclaration(node)) return "enum"; if (ts.isMethodDeclaration(node)) return "method"; if (ts.isConstructorDeclaration(node)) return "constructor"; return "variable"; }
function owners(node: ts.Node): string[] { const values: string[] = []; for (let parent = node.parent; parent; parent = parent.parent) if (named(parent)) values.unshift(nameOf(parent)); return values; }

export function extractDeclarations(program: ts.Program, root: string, ownedFiles: Set<string>): Declarations {
  const checker = program.getTypeChecker(); const entities: Entity[] = []; const nodes = new Map<ts.Node, string>(); const symbols = new Map<ts.Symbol, string>(); const counts = new Map<string, number>();
  for (const source of program.getSourceFiles()) {
    if (!ownedFiles.has(ts.sys.resolvePath(source.fileName)) || source.fileName.includes("node_modules")) continue;
    const moduleFile = relativePath(root, source.fileName);
    entities.push({ name: "<module>", ownership_chain: "<module>", declaration_kind: "module", file: moduleFile, discriminator: 0, start_line: 1, start_column: 1, end_line: source.getLineAndCharacterOfPosition(source.end).line + 1, end_column: 1, attributes: {} });
    const visit = (node: ts.Node) => {
      if (named(node)) {
        const name = nameOf(node); const ownership_chain = [...owners(node), name].join("."); const declaration_kind = kindOf(node); const file = relativePath(root, source.fileName); const group = `${file}#${ownership_chain}#${declaration_kind}`; const discriminator = counts.get(group) ?? 0; counts.set(group, discriminator + 1); const key = `${group}#${discriminator}`; const span = spanOf(root, node);
        entities.push({ name, ownership_chain, declaration_kind, file, discriminator, start_line: span.start_line, start_column: span.start_column, end_line: span.end_line, end_column: span.end_column, attributes: { implementation: "body" in node && !!node.body } });
        nodes.set(node, key); const symbol = checker.getSymbolAtLocation(node.name); if (symbol) symbols.set(symbol, key);
      }
      ts.forEachChild(node, visit);
    }; visit(source);
  }
  return { entities, nodes, symbols };
}

export function enclosingKey(node: ts.Node, declarations: Declarations): string | undefined { for (let current: ts.Node | undefined = node; current; current = current.parent) { const key = declarations.nodes.get(current); if (key) return key; } return undefined; }
export function resolvedSymbol(checker: ts.TypeChecker, symbol: ts.Symbol | undefined): ts.Symbol | undefined { const seen = new Set<ts.Symbol>(); while (symbol && (symbol.flags & ts.SymbolFlags.Alias) && !seen.has(symbol)) { seen.add(symbol); symbol = checker.getAliasedSymbol(symbol); } return symbol; }
export function symbolKey(checker: ts.TypeChecker, symbol: ts.Symbol | undefined, declarations: Declarations): string | undefined { symbol = resolvedSymbol(checker, symbol); if (!symbol) return undefined; const direct = declarations.symbols.get(symbol); if (direct) return direct; for (const declaration of symbol.declarations ?? []) { const key = declarations.nodes.get(declaration); if (key) return key; } return undefined; }
