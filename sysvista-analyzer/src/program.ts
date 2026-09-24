import { existsSync, readFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import ts from "typescript";
import { LIBRARIES } from "embedded-libs";
import type { Diagnostic } from "./contract.js";
import { relativePath } from "./spans.js";

/**
 * One TypeScript program. `ownedFiles` are the scanned files this program emits
 * entities for; `scannedFiles` are every scanned file, so declarations in scanned
 * files that another program owns can still be resolved to their canonical keys.
 */
export interface Project { config?: string; program: ts.Program; ownedFiles: Set<string>; scannedFiles: Set<string> }
export interface Projects { projects: Project[]; diagnostics: Diagnostic[] }

/** Options for scanned files that no tsconfig includes. Real configs are used as written. */
const FALLBACK_OPTIONS: ts.CompilerOptions = { noEmit: true, skipLibCheck: true, allowJs: true, moduleResolution: ts.ModuleResolutionKind.NodeNext, module: ts.ModuleKind.NodeNext, target: ts.ScriptTarget.ES2022 };
const NO_INPUTS = 18003;

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

/**
 * Expand a tsconfig and everything it references into the configs that actually
 * compile files. A solution-style config (`files: []` plus `references`) compiles
 * nothing itself; each referenced config is its own project, as with `tsc -b`.
 */
function expandConfig(path: string, root: string, seen: Set<string>, out: Map<string, ts.ParsedCommandLine>, diagnostics: Diagnostic[]): void {
  const config = ts.sys.resolvePath(path);
  if (seen.has(config)) return;
  seen.add(config);
  if (!existsSync(config)) {
    diagnostics.push({ message: `tsconfig ${relativePath(root, config)} is referenced but does not exist`, severity: "warning" });
    return;
  }
  const loaded = ts.readConfigFile(config, ts.sys.readFile);
  if (loaded.error) {
    diagnostics.push(configDiagnostic(root, config, loaded.error));
    return;
  }
  const parsed = ts.parseJsonConfigFileContent(loaded.config, ts.sys, dirname(config), undefined, config);
  const references = parsed.projectReferences ?? [];
  for (const error of parsed.errors) {
    if (error.code === NO_INPUTS && references.length) continue;
    diagnostics.push(configDiagnostic(root, config, error));
  }
  if (parsed.fileNames.length) out.set(config, parsed);
  for (const reference of references) expandConfig(ts.resolveProjectReferencePath(reference), root, seen, out, diagnostics);
}

export function createProjects(root: string, files: string[], explicitConfig?: string): Projects {
  const scannedFiles = new Set(files.map(file => ts.sys.resolvePath(resolve(root, file))));
  const diagnostics: Diagnostic[] = [];
  const entryConfigs = explicitConfig
    ? [resolve(root, explicitConfig)]
    : [...new Set([...scannedFiles].map(file => nearestTsconfig(file, root)).filter((config): config is string => !!config))];
  const configs = new Map<string, ts.ParsedCommandLine>();
  const seen = new Set<string>();
  for (const config of entryConfigs.sort()) expandConfig(config, root, seen, configs, diagnostics);

  const workspaces = discoverWorkspacePackages(root);
  const owners = new Map<string, string[]>();
  const projects: Project[] = [];
  for (const [config, parsed] of [...configs].sort(([a], [b]) => compare(a, b))) {
    const ownedFiles = new Set(parsed.fileNames.map(file => ts.sys.resolvePath(file)).filter(file => scannedFiles.has(file)));
    if (!ownedFiles.size) continue;
    for (const file of ownedFiles) owners.set(file, [...(owners.get(file) ?? []), config]);
    projects.push(project(root, config, parsed.fileNames, { ...parsed.options, noEmit: true, skipLibCheck: true }, ownedFiles, scannedFiles, workspaces));
  }
  const orphans = [...scannedFiles].filter(file => !owners.has(file)).sort(compare);
  if (orphans.length) projects.push(project(root, undefined, orphans, FALLBACK_OPTIONS, new Set(orphans), scannedFiles, workspaces));

  // Overlapping includes are analysed once per project and merged by canonical key.
  for (const [file, configsForFile] of [...owners].sort(([a], [b]) => compare(a, b))) {
    if (configsForFile.length < 2) continue;
    const path = relativePath(root, file);
    diagnostics.push({
      message: `${path} is included by ${configsForFile.length} tsconfig projects (${configsForFile.map(config => relativePath(root, config)).join(", ")}); it was analysed by each and the results merged`,
      severity: "warning",
      span: { file: path, start_line: 1, start_column: 1, end_line: 1, end_column: 1 },
    });
  }
  return { projects, diagnostics };
}

function project(root: string, config: string | undefined, rootNames: string[], parsedOptions: ts.CompilerOptions, ownedFiles: Set<string>, scannedFiles: Set<string>, workspaces: Map<string, string>): Project {
  let options = parsedOptions;
  if (workspaces.size) {
    const paths = { ...(options.paths ?? {}) };
    for (const [name, target] of workspaces) paths[name] ??= [target];
    options = { ...options, baseUrl: options.baseUrl ?? root, paths };
  }
  return { config: config && relativePath(root, config), program: ts.createProgram({ rootNames, options, host: compilerHost(options) }), ownedFiles, scannedFiles };
}

function configDiagnostic(root: string, config: string, diagnostic: ts.Diagnostic): Diagnostic {
  return { message: `${relativePath(root, config)}: ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")}`, severity: diagnostic.category === ts.DiagnosticCategory.Error ? "error" : "warning" };
}

/** Code-point order; `localeCompare` varies by locale. */
export function compare(a: string, b: string): number { return a < b ? -1 : a > b ? 1 : 0; }

/** Directory the compiler looks in for its default libraries, which the bundle embeds. */
export function libraryDirectory(options: ts.CompilerOptions = FALLBACK_OPTIONS): string {
  return dirname(ts.sys.resolvePath(ts.getDefaultLibFilePath(options)));
}

function compilerHost(options: ts.CompilerOptions): ts.CompilerHost {
  const host = ts.createCompilerHost(options);
  const libraries = libraryDirectory(options);
  const embedded = (fileName: string): string | undefined => dirname(ts.sys.resolvePath(fileName)) === libraries ? LIBRARIES[basename(fileName)] : undefined;
  const getSourceFile = host.getSourceFile.bind(host);
  const fileExists = host.fileExists.bind(host);
  const readFile = host.readFile.bind(host);
  host.fileExists = fileName => embedded(fileName) !== undefined || fileExists(fileName);
  host.readFile = fileName => embedded(fileName) ?? readFile(fileName);
  host.getSourceFile = (fileName, languageVersion, onError, shouldCreateNewSourceFile) => {
    const text = embedded(fileName);
    return text === undefined ? getSourceFile(fileName, languageVersion, onError, shouldCreateNewSourceFile) : ts.createSourceFile(fileName, text, languageVersion, true);
  };
  return host;
}
