import { FolderOpen, Upload } from "lucide-react";
import { useEffect, useRef } from "react";
import type { LoadedSnapshot } from "../../lib/loader";
import { useBundle } from "../../hooks/useBundle";

interface ImportDialogProps {
  onLoad: (data: LoadedSnapshot) => void;
  onError: (message: string) => void;
}

export function ImportDialog({ onLoad, onError }: ImportDialogProps) {
  const file = useRef<HTMLInputElement>(null);
  const directory = useRef<HTMLInputElement>(null);
  const { importFiles, isDragging } = useBundle(onLoad, onError);
  useEffect(() => { directory.current?.setAttribute("webkitdirectory", ""); }, []);
  const change = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.files?.length) {
      onError("");
      void importFiles(event.target.files);
    }
    event.target.value = "";
  };
  return <>
    {isDragging && <div className="pointer-events-none fixed inset-3 z-50 grid place-items-center rounded-xl border-2 border-dashed border-sky-500 bg-sky-50/90 text-lg font-semibold text-sky-800 dark:bg-slate-950/90 dark:text-sky-200">Drop SysVista JSON, zip, or bundle files to load</div>}
    <button className="toolbar-button" onClick={() => file.current?.click()}><Upload className="h-4 w-4" /> Import</button>
    <button className="toolbar-button" onClick={() => directory.current?.click()}><FolderOpen className="h-4 w-4" /> Bundle folder</button>
    <input ref={file} hidden type="file" accept=".json,.zip,application/zip" onChange={change} />
    <input ref={directory} hidden type="file" multiple onChange={change} />
  </>;
}
