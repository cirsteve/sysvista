import type { SysVistaOutput } from "../types/schema";

function validate(data: unknown): SysVistaOutput {
  const obj = data as Record<string, unknown>;
  if (
    !obj ||
    typeof obj !== "object" ||
    !Array.isArray(obj.components) ||
    !Array.isArray(obj.edges)
  ) {
    throw new Error(
      "Invalid SysVista JSON: missing required fields (components, edges)",
    );
  }
  return obj as unknown as SysVistaOutput;
}

export async function loadFromFile(file: File): Promise<SysVistaOutput> {
  const text = await file.text();
  const parsed = JSON.parse(text);
  return validate(parsed);
}

export async function loadFromUrl(url: string): Promise<SysVistaOutput> {
  const resp = await fetch(url);
  if (!resp.ok) throw new Error(`Failed to fetch: ${resp.status}`);
  const data = await resp.json();
  return validate(data);
}

export function setupDropZone(
  element: HTMLElement,
  onLoad: (data: SysVistaOutput) => void,
  onError: (message: string) => void,
  onDragStateChange: (isDragging: boolean) => void,
) {
  let dragCounter = 0;

  const handleDragEnter = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter++;
    if (dragCounter === 1) onDragStateChange(true);
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter--;
    if (dragCounter === 0) onDragStateChange(false);
  };

  const handleDrop = async (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter = 0;
    onDragStateChange(false);
    const file = e.dataTransfer?.files[0];
    if (!file) return;
    if (!file.name.endsWith(".json")) {
      onError("Please drop a .json file");
      return;
    }
    try {
      const data = await loadFromFile(file);
      onLoad(data);
    } catch (err) {
      onError(err instanceof Error ? err.message : "Failed to load file");
    }
  };

  element.addEventListener("dragenter", handleDragEnter);
  element.addEventListener("dragover", handleDragOver);
  element.addEventListener("dragleave", handleDragLeave);
  element.addEventListener("drop", handleDrop);

  return () => {
    element.removeEventListener("dragenter", handleDragEnter);
    element.removeEventListener("dragover", handleDragOver);
    element.removeEventListener("dragleave", handleDragLeave);
    element.removeEventListener("drop", handleDrop);
  };
}
