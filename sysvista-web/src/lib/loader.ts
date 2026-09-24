import type { SysVistaOutput } from "../types/schema";
import type { Diagnostic, FileId, Snapshot } from "../types/v2";
import { adaptV1 } from "./adapters/v1";
import { DEFAULT_ARCHIVE_TOTAL_CAP, openBundleArchive } from "./bundle/archive";
import type { Result } from "./result";
import { validateReferences, type ReferenceError } from "./validate/references";
import { validateSnapshot, type ValidationError } from "./validate/v2";

export interface SourceIndexEntry {
  file_id: FileId;
  path: string;
  content_hash?: string;
  byte_length?: number;
  source_available: boolean;
}

export interface BundleSourceStore {
  included: boolean;
  index: ReadonlyMap<FileId, SourceIndexEntry>;
  read(contentHash: string): Promise<Uint8Array | undefined>;
}

export interface LoadedSnapshot {
  snapshot: Snapshot;
  origin: "v1-legacy" | "v2";
  sources?: BundleSourceStore;
}
export type LoadError =
  | { kind: "parse" | "fetch" | "format"; message: string }
  | { kind: "validation"; message: string; errors: ValidationError[] }
  | { kind: "references"; message: string; errors: ReferenceError[] };

const failure = (error: LoadError): Result<LoadedSnapshot, LoadError> => ({ ok: false, error });
const isRecord = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);
const isV1 = (value: unknown): value is SysVistaOutput => isRecord(value) && Array.isArray(value.components) && Array.isArray(value.edges);
const bundleSnapshot = (value: Record<string, unknown>): unknown =>
  isRecord(value.manifest) && isRecord(value.graph)
    ? { ...value.graph, manifest: value.manifest, diagnostics: value.diagnostics ?? [], findings: value.findings ?? value.graph.findings ?? [] }
    : value;

export function validate(data: unknown): Result<LoadedSnapshot, LoadError> {
  if (isV1(data)) {
    const snapshot = adaptV1({ ...data, workflows: Array.isArray(data.workflows) ? data.workflows : [] });
    const refs = validateReferences(snapshot);
    return refs.ok ? { ok: true, value: { snapshot, origin: "v1-legacy" } }
      : failure({ kind: "references", message: "Legacy input contains dangling references", errors: refs.error });
  }
  if (!isRecord(data)) return failure({ kind: "format", message: "Invalid SysVista JSON: expected an object" });
  const schemaResult = validateSnapshot(bundleSnapshot(data));
  if (!schemaResult.ok) return failure({ kind: "validation", message: "Invalid SysVista v2 snapshot", errors: schemaResult.error });
  const refs = validateReferences(schemaResult.value);
  return refs.ok ? { ok: true, value: { snapshot: refs.value, origin: "v2" } }
    : failure({ kind: "references", message: "Snapshot contains dangling references", errors: refs.error });
}

const parse = (text: string): Result<LoadedSnapshot, LoadError> => {
  try { return validate(JSON.parse(text)); }
  catch (cause) { return failure({ kind: "parse", message: cause instanceof Error ? cause.message : "Invalid JSON" }); }
};

export async function loadFromFile(file: File): Promise<Result<LoadedSnapshot, LoadError>> {
  try {
    if (file.name.toLowerCase().endsWith(".zip")) {
      if (file.size > DEFAULT_ARCHIVE_TOTAL_CAP) {
        return failure({ kind: "format", message: `Archive is ${file.size} bytes; cap is ${DEFAULT_ARCHIVE_TOTAL_CAP} bytes` });
      }
      return loadFromArchive(new Uint8Array(await file.arrayBuffer()));
    }
    return parse(await file.text());
  } catch (cause) {
    return failure({ kind: "parse", message: cause instanceof Error ? cause.message : "Failed to read file" });
  }
}

const decodeJson = (bytes: Uint8Array | undefined, name: string): unknown => {
  if (!bytes) throw new Error(`Incomplete SysVista v2 bundle: missing ${name}`);
  return JSON.parse(new TextDecoder().decode(bytes));
};

const sourceIndex = (value: unknown): Map<FileId, SourceIndexEntry> => {
  if (!isRecord(value)) return new Map();
  const files = value.files;
  if (!Array.isArray(files)) return new Map();
  return new Map(files.filter(isRecord).flatMap((item) =>
    typeof item.file_id === "string" && typeof item.path === "string" && typeof item.source_available === "boolean"
      ? [[item.file_id as FileId, item as unknown as SourceIndexEntry] as const]
      : [],
  ));
};

export async function loadFromArchive(bytes: Uint8Array): Promise<Result<LoadedSnapshot, LoadError>> {
  try {
    const archive = await openBundleArchive(bytes);
    const manifest = decodeJson(archive.metadata.get("manifest.json"), "manifest.json");
    const graph = decodeJson(archive.metadata.get("graph.json"), "graph.json");
    const diagnostics = decodeJson(archive.metadata.get("diagnostics.json"), "diagnostics.json");
    const findings = archive.metadata.has("findings.json") ? decodeJson(archive.metadata.get("findings.json"), "findings.json") : undefined;
    const scopes = decodeJson(archive.metadata.get("index/scopes.json"), "index/scopes.json");
    const index = sourceIndex(archive.metadata.has("source-index.json")
      ? decodeJson(archive.metadata.get("source-index.json"), "source-index.json")
      : { files: [] });
    const result = validate({ manifest, graph, diagnostics, findings });
    if (!result.ok) return result;
    result.value.snapshot.scope_index = scopes;
    result.value.snapshot.diagnostics = [
      ...(result.value.snapshot.diagnostics ?? []),
      ...archive.diagnostics,
    ] as Diagnostic[];
    const included = isRecord(manifest) && manifest.source_included !== false;
    result.value.sources = {
      included,
      index,
      read: (hash) => archive.read(`source/${hash}`),
    };
    return result;
  } catch (cause) {
    return failure({ kind: "parse", message: cause instanceof Error ? cause.message : "Failed to read archive" });
  }
}

const bundlePart = (files: File[], suffix: string) =>
  files.find((file) => file.name === suffix || file.webkitRelativePath.endsWith(`/${suffix}`));

export async function loadFromFiles(files: Iterable<File>): Promise<Result<LoadedSnapshot, LoadError>> {
  const items = [...files];
  if (items.length === 1) return loadFromFile(items[0]);
  const manifest = bundlePart(items, "manifest.json");
  const graph = bundlePart(items, "graph.json");
  const diagnostics = bundlePart(items, "diagnostics.json");
  const findings = bundlePart(items, "findings.json");
  const scopes = bundlePart(items, "scopes.json");
  const sourceIndexFile = bundlePart(items, "source-index.json");
  const missing = [
    ["manifest.json", manifest], ["graph.json", graph],
    ["diagnostics.json", diagnostics], ["index/scopes.json", scopes],
  ].filter(([, file]) => !file).map(([name]) => name);
  if (missing.length > 0) {
    return failure({ kind: "format", message: `Incomplete SysVista v2 bundle: missing ${missing.join(", ")}` });
  }
  if (!manifest || !graph || !diagnostics || !scopes) {
    return failure({ kind: "format", message: "Incomplete SysVista v2 bundle" });
  }
  try {
    const [manifestValue, graphValue, diagnosticsValue, scopeIndex] = await Promise.all(
      [manifest, graph, diagnostics, scopes].map(async (file) => JSON.parse(await file.text())),
    );
    const findingsValue = findings ? JSON.parse(await findings.text()) : undefined;
    const result = validate({ manifest: manifestValue, graph: graphValue, diagnostics: diagnosticsValue, findings: findingsValue });
    if (result.ok) {
      result.value.snapshot.scope_index = scopeIndex;
      const index = sourceIndex(sourceIndexFile ? JSON.parse(await sourceIndexFile.text()) : { files: [] });
      const sourceFiles = new Map(items.map((file) => [file.webkitRelativePath || file.name, file]));
      const findSource = (hash: string) => [...sourceFiles].find(([path]) => path === `source/${hash}` || path.endsWith(`/source/${hash}`))?.[1];
      result.value.sources = {
        included: isRecord(manifestValue) && manifestValue.source_included !== false,
        index,
        read: async (hash) => {
          const source = findSource(hash);
          return source ? new Uint8Array(await source.arrayBuffer()) : undefined;
        },
      };
    }
    return result;
  } catch (cause) {
    return failure({ kind: "parse", message: cause instanceof Error ? cause.message : "Failed to read v2 bundle" });
  }
}

export async function loadFromUrl(url: string): Promise<Result<LoadedSnapshot, LoadError>> {
  try {
    const response = await fetch(url);
    if (!response.ok) return failure({ kind: "fetch", message: `Failed to fetch: ${response.status}` });
    return parse(await response.text());
  } catch (cause) {
    return failure({ kind: "fetch", message: cause instanceof Error ? cause.message : "Failed to fetch" });
  }
}

export function formatLoadError(error: LoadError): string {
  return "errors" in error ? `${error.message}: ${error.errors.map((item) => item.message).join("; ")}` : error.message;
}

export function setupDropZone(element: HTMLElement, onLoad: (data: LoadedSnapshot) => void, onError: (message: string) => void, onDragStateChange: (isDragging: boolean) => void) {
  let dragCounter = 0;
  const enter = (event: DragEvent) => { event.preventDefault(); event.stopPropagation(); if (++dragCounter === 1) onDragStateChange(true); };
  const over = (event: DragEvent) => { event.preventDefault(); event.stopPropagation(); };
  const leave = (event: DragEvent) => { event.preventDefault(); event.stopPropagation(); if (--dragCounter === 0) onDragStateChange(false); };
  const drop = async (event: DragEvent) => {
    event.preventDefault(); event.stopPropagation(); dragCounter = 0; onDragStateChange(false);
    const file = event.dataTransfer?.files[0];
    if (!file) return;
    if (!file.name.endsWith(".json") && !file.name.endsWith(".zip")) { onError("Please drop a .json or .zip file"); return; }
    const result = await loadFromFiles(event.dataTransfer?.files ?? [file]);
    if (result.ok) onLoad(result.value); else onError(formatLoadError(result.error));
  };
  element.addEventListener("dragenter", enter); element.addEventListener("dragover", over);
  element.addEventListener("dragleave", leave); element.addEventListener("drop", drop);
  return () => {
    element.removeEventListener("dragenter", enter); element.removeEventListener("dragover", over);
    element.removeEventListener("dragleave", leave); element.removeEventListener("drop", drop);
  };
}

export { adaptV1, validateReferences, validateSnapshot };
