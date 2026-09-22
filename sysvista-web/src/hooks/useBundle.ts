import { useCallback, useEffect, useState } from "react";
import { formatLoadError, loadFromFiles, setupDropZone, type LoadedSnapshot } from "../lib/loader";

export function useBundle(onLoad: (data: LoadedSnapshot) => void, onError: (message: string) => void) {
  const [isDragging, setIsDragging] = useState(false);
  const importFiles = useCallback(async (files: Iterable<File>) => {
    const result = await loadFromFiles(files);
    if (result.ok) onLoad(result.value);
    else onError(formatLoadError(result.error));
  }, [onError, onLoad]);

  useEffect(() => setupDropZone(document.body, onLoad, onError, setIsDragging), [onError, onLoad]);
  return { importFiles, isDragging };
}
