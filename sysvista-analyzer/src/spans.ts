import ts from "typescript";
import type { Span } from "./contract.js";

export function relativePath(root: string, file: string): string {
  const normalizedRoot = ts.sys.resolvePath(root).replaceAll("\\", "/").replace(/\/$/, "");
  const normalized = ts.sys.resolvePath(file).replaceAll("\\", "/");
  return normalized.startsWith(`${normalizedRoot}/`) ? normalized.slice(normalizedRoot.length + 1) : normalized;
}

export function spanOf(root: string, node: ts.Node): Span {
  const source = node.getSourceFile();
  const start = source.getLineAndCharacterOfPosition(node.getStart(source, false));
  const end = source.getLineAndCharacterOfPosition(node.getEnd());
  return { file: relativePath(root, source.fileName), start_line: start.line + 1, start_column: start.character + 1, end_line: end.line + 1, end_column: end.character + 1 };
}
