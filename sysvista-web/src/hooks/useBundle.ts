import { useCallback, useEffect, useRef, useState } from "react";
import { formatLoadError, loadFromFiles, setupDropZone, type LoadedSnapshot } from "../lib/loader";

export function useBundle(onLoad: (data: LoadedSnapshot) => void, onError: (message: string) => void) {
  const [isDragging, setIsDragging] = useState(false);
  const callbacks = useRef({ onLoad, onError });
  useEffect(() => { callbacks.current = { onLoad, onError }; }, [onLoad, onError]);
  const importFiles = useCallback(async (files: Iterable<File>) => {
    const result = await loadFromFiles(files);
    if (result.ok) onLoad(result.value);
    else onError(formatLoadError(result.error));
  }, [onError, onLoad]);

  useEffect(() => setupDropZone(document.body,
    (data) => callbacks.current.onLoad(data),
    (message) => callbacks.current.onError(message), setIsDragging), []);
  return { importFiles, isDragging };
}
