import type { SysVistaOutput } from "../types/schema";
import type { Snapshot } from "../types/v2";
import { adaptV1 } from "./adapters/v1";
import type { Result } from "./result";
import { validateReferences, type ReferenceError } from "./validate/references";
import { validateSnapshot, type ValidationError } from "./validate/v2";

export interface LoadedSnapshot { snapshot: Snapshot; origin: "v1-legacy" | "v2" }
export type LoadError =
  | { kind: "parse" | "fetch" | "format"; message: string }
  | { kind: "validation"; message: string; errors: ValidationError[] }
  | { kind: "references"; message: string; errors: ReferenceError[] };

const failure = (error: LoadError): Result<LoadedSnapshot, LoadError> => ({ ok: false, error });
const isRecord = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);
const isV1 = (value: unknown): value is SysVistaOutput => isRecord(value) && Array.isArray(value.components) && Array.isArray(value.edges);
const bundleSnapshot = (value: Record<string, unknown>): unknown =>
  isRecord(value.manifest) && isRecord(value.graph)
    ? { ...value.graph, manifest: value.manifest, diagnostics: value.diagnostics ?? [] }
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
    return parse(await file.text());
  } catch (cause) {
    return failure({ kind: "parse", message: cause instanceof Error ? cause.message : "Failed to read file" });
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
  const scopes = bundlePart(items, "scopes.json");
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
    const result = validate({ manifest: manifestValue, graph: graphValue, diagnostics: diagnosticsValue });
    if (result.ok) result.value.snapshot.scope_index = scopeIndex;
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
    if (!file.name.endsWith(".json")) { onError("Please drop a .json file"); return; }
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
