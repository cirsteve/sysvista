import { Unzip, UnzipInflate } from "fflate";
import type { Diagnostic } from "../../types/v2";
import { validateEntryPath } from "./paths";

export const DEFAULT_ARCHIVE_ENTRY_CAP = 2 * 1024 * 1024;
export const DEFAULT_ARCHIVE_TOTAL_CAP = 512 * 1024 * 1024;
export const DEFAULT_ARCHIVE_ENTRY_COUNT_CAP = 10_000;

const warning = (message: string): Diagnostic => ({ kind: "warning", message });

export interface BundleArchive {
  entries: readonly string[];
  diagnostics: Diagnostic[];
  read(path: string): Promise<Uint8Array | undefined>;
}

interface ScanResult {
  entries: string[];
  diagnostics: Diagnostic[];
  metadata: Map<string, Uint8Array>;
  value?: Uint8Array;
}

/**
 * fflate's streaming reader exposes name and uncompressed size before start().
 * No entry is started until its path and per-entry, aggregate, and count limits
 * pass; a later scan reads one requested source without retaining every source.
 */
function scan(bytes: Uint8Array, cap: number, totalCap: number, entryCountCap: number, requested?: string): Promise<ScanResult> {
  return new Promise((resolve, reject) => {
    const entries: string[] = [];
    const diagnostics: Diagnostic[] = [];
    const metadata = new Map<string, Uint8Array>();
    let value: Uint8Array | undefined;
    let pending = 0;
    let pushed = false;
    let encountered = 0;
    let acceptedBytes = 0;
    const finish = () => { if (pushed && pending === 0) resolve({ entries, diagnostics, metadata, value }); };
    const unzip = new Unzip((entry) => {
      encountered += 1;
      if (encountered > entryCountCap) {
        if (encountered === entryCountCap + 1) diagnostics.push(warning(`Rejected remaining archive entries: ${entryCountCap} entry cap exceeded`));
        return;
      }
      const validated = validateEntryPath(entry.name);
      if (!validated.ok) {
        diagnostics.push(warning(`Rejected archive entry '${entry.name}': ${validated.reason}`));
        return;
      }
      if (entry.originalSize === undefined) {
        diagnostics.push(warning(`Rejected archive entry '${entry.name}': uncompressed size is unavailable`));
        return;
      }
      if (entry.originalSize > cap) {
        diagnostics.push(warning(`Rejected archive entry '${entry.name}': ${entry.originalSize} bytes exceeds ${cap} byte cap`));
        return;
      }
      if (acceptedBytes + entry.originalSize > totalCap) {
        diagnostics.push(warning(`Rejected archive entry '${entry.name}': total uncompressed size exceeds ${totalCap} byte cap`));
        return;
      }
      acceptedBytes += entry.originalSize;
      entries.push(validated.path);
      const shouldReadMetadata = requested === undefined && !entry.name.startsWith("source/");
      if (!shouldReadMetadata && entry.name !== requested) return;
      pending += 1;
      const chunks: Uint8Array[] = [];
      let length = 0;
      entry.ondata = (error, chunk, final) => {
        if (error) { reject(error); return; }
        length += chunk.length;
        if (length > cap) {
          entry.terminate();
          diagnostics.push(warning(`Rejected archive entry '${entry.name}': expanded data exceeds ${cap} byte cap`));
          pending -= 1;
          finish();
          return;
        }
        chunks.push(chunk);
        if (!final) return;
        const joined = new Uint8Array(length);
        let offset = 0;
        chunks.forEach((part) => { joined.set(part, offset); offset += part.length; });
        if (entry.name === requested) value = joined;
        else metadata.set(entry.name, joined);
        pending -= 1;
        finish();
      };
      entry.start();
    });
    unzip.register(UnzipInflate);
    try {
      unzip.push(bytes, true);
      pushed = true;
      finish();
    } catch (cause) {
      reject(cause);
    }
  });
}

export interface OpenedBundleArchive extends BundleArchive {
  metadata: ReadonlyMap<string, Uint8Array>;
}

export async function openBundleArchive(
  bytes: Uint8Array,
  cap = DEFAULT_ARCHIVE_ENTRY_CAP,
  totalCap = DEFAULT_ARCHIVE_TOTAL_CAP,
  entryCountCap = DEFAULT_ARCHIVE_ENTRY_COUNT_CAP,
): Promise<OpenedBundleArchive> {
  const initial = await scan(bytes, cap, totalCap, entryCountCap);
  const allowed = new Set(initial.entries);
  return {
    entries: initial.entries,
    diagnostics: initial.diagnostics,
    metadata: initial.metadata,
    async read(path) {
      if (!allowed.has(path)) return undefined;
      return (await scan(bytes, cap, totalCap, entryCountCap, path)).value;
    },
  };
}
