import { useEffect, useState } from "react";
import { decodeSource } from "../lib/bundle/source";
import type { LoadedSnapshot } from "../lib/loader";
import type { FileId } from "../types/v2";

export type SourceResult =
  | { kind: "idle" | "loading" | "unavailable" | "binary" }
  | { kind: "text"; text: string };

export function useSource(loaded: LoadedSnapshot | null, fileId?: FileId): SourceResult {
  const [result, setResult] = useState<SourceResult>({ kind: "idle" });
  useEffect(() => {
    let active = true;
    const sources = loaded?.sources;
    const indexed = fileId ? sources?.index.get(fileId) : undefined;
    if (!fileId) { setResult({ kind: "idle" }); return; }
    if (!sources?.included || !indexed?.source_available || !indexed.content_hash) {
      setResult({ kind: "unavailable" });
      return;
    }
    setResult({ kind: "loading" });
    void sources.read(indexed.content_hash).then((bytes) => {
      if (!active) return;
      if (!bytes) { setResult({ kind: "unavailable" }); return; }
      const decoded = decodeSource(bytes);
      setResult(decoded.kind === "binary" ? { kind: "binary" } : decoded);
    }, () => active && setResult({ kind: "unavailable" }));
    return () => { active = false; };
  }, [fileId, loaded]);
  return result;
}
