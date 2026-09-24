import { useEffect, useState } from "react";
import { decodeSource } from "../lib/bundle/source";
import type { LoadedSnapshot } from "../lib/loader";
import type { FileId } from "../types/v2";

export type SourceResult =
  | { kind: "idle" | "loading" | "unavailable" | "binary" }
  | { kind: "text"; text: string };

export function useSource(loaded: LoadedSnapshot | null, fileId?: FileId): SourceResult {
  const sources = loaded?.sources;
  const indexed = fileId ? sources?.index.get(fileId) : undefined;
  const requestKey = sources?.included && indexed?.source_available && indexed.content_hash ? indexed.content_hash : undefined;
  const [resolved, setResolved] = useState<{ key: string; result: SourceResult }>();
  useEffect(() => {
    let active = true;
    if (!requestKey || !sources) return;
    void sources.read(requestKey).then((bytes) => {
      if (!active) return;
      if (!bytes) { setResolved({ key: requestKey, result: { kind: "unavailable" } }); return; }
      const decoded = decodeSource(bytes);
      setResolved({ key: requestKey, result: decoded.kind === "binary" ? { kind: "binary" } : decoded });
    }, () => active && setResolved({ key: requestKey, result: { kind: "unavailable" } }));
    return () => { active = false; };
  }, [requestKey, sources]);
  if (!fileId) return { kind: "idle" };
  if (!requestKey) return { kind: "unavailable" };
  return resolved?.key === requestKey ? resolved.result : { kind: "loading" };
}
