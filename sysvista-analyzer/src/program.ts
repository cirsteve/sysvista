import { existsSync, readFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import ts from "typescript";
import { LIBRARIES } from "embedded-libs";

export interface Project { config?: string; program: ts.Program; ownedFiles: Set<string> }

export function nearestTsconfig(file: string, root: string): string | undefined {
  let directory = dirname(resolve(file));
  const boundary = resolve(root);
  while (directory.startsWith(boundary)) {
    const candidate = join(directory, "tsconfig.json");
    if (existsSync(candidate)) return candidate;
    if (directory === boundary) break;
    directory = dirname(directory);
  }
  return undefined;
}

export function discoverWorkspacePackages(root: string): Map<string, string> {
  const result = new Map<string, string>();
  const manifest = join(root, "package.json");
  if (!existsSync(manifest)) return result;
  try {
    const value = JSON.parse(readFileSync(manifest, "utf8")) as { workspaces?: string[] | { packages?: string[] } };
    const patterns = Array.isArray(value.workspaces) ? value.workspaces : value.workspaces?.packages ?? [];
    for (const pattern of patterns) {
      for (const packageJson of ts.sys.readDirectory(root, [".json"], undefined, [`${pattern}/package.json`])) {
        try {
          const pkg = JSON.parse(readFileSync(packageJson, "utf8")) as { name?: string; types?: string; main?: string };
          if (pkg.name) result.set(pkg.name, resolve(dirname(packageJson), pkg.types ?? pkg.main ?? "src/index.ts"));
        } catch { /* malformed workspace manifests become compiler diagnostics later */ }
      }
    }
  } catch { /* root package diagnostics are reported by the caller's programs */ }
  return result;
}

export function createProjects(root: string, files: string[], explicitConfig?: string): Project[] {
  const absolute = files.map(file => resolve(root, file));
  const groups = new Map<string, string[]>();
  for (const file of absolute) {
    const config = explicitConfig ? resolve(root, explicitConfig) : nearestTsconfig(file, root);
    const key = config ?? "<orphans>";
    groups.set(key, [...(groups.get(key) ?? []), file]);
  }
  const workspaces = discoverWorkspacePackages(root);
  return [...groups.entries()].map(([key, owned]) => {
    let options: ts.CompilerOptions = { noEmit: true, skipLibCheck: true, allowJs: true, checkJs: true, moduleResolution: ts.ModuleResolutionKind.NodeNext, module: ts.ModuleKind.NodeNext, target: ts.ScriptTarget.ES2022 };
    let roots = owned;
    if (key !== "<orphans>") {
      const loaded = ts.readConfigFile(key, ts.sys.readFile);
      if (!loaded.error) {
        const parsed = ts.parseJsonConfigFileContent(loaded.config, ts.sys, dirname(key));
        options = { ...parsed.options, noEmit: true, skipLibCheck: true };
        roots = [...new Set([...parsed.fileNames, ...owned])];
      }
    }
    const paths = { ...(options.paths ?? {}) };
    for (const [name, target] of workspaces) paths[name] ??= [target];
    if (workspaces.size) options = { ...options, baseUrl: options.baseUrl ?? root, paths };
    return { config: key === "<orphans>" ? undefined : key, program: ts.createProgram({ rootNames: roots, options, host: compilerHost(options) }), ownedFiles: new Set(owned.map(ts.sys.resolvePath)) };
  });
}

function compilerHost(options: ts.CompilerOptions): ts.CompilerHost {
  const host = ts.createCompilerHost(options);
  const getSourceFile = host.getSourceFile.bind(host);
  const fileExists = host.fileExists.bind(host);
  const readFile = host.readFile.bind(host);
  host.fileExists = fileName => basename(fileName) in LIBRARIES || fileExists(fileName);
  host.readFile = fileName => LIBRARIES[basename(fileName)] ?? readFile(fileName);
  host.getSourceFile = (fileName, languageVersion, onError, shouldCreateNewSourceFile) => {
    const embedded = LIBRARIES[basename(fileName)];
    return embedded === undefined ? getSourceFile(fileName, languageVersion, onError, shouldCreateNewSourceFile) : ts.createSourceFile(fileName, embedded, languageVersion, true);
  };
  return host;
}
