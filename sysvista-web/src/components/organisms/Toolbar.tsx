import { FolderOpen, Upload } from "lucide-react";
import { useEffect, useRef } from "react";
import type { LoadedSnapshot } from "../../lib/loader";
import { formatLoadError, loadFromFiles } from "../../lib/loader";
import { ThemeToggle } from "../atoms/ThemeToggle";

interface ToolbarProps {
  projectName?: string;
  theme: "light" | "dark";
  onToggleTheme: () => void;
  onLoad: (data: LoadedSnapshot) => void;
  onError: (message: string) => void;
}

export function Toolbar({ projectName, theme, onToggleTheme, onLoad, onError }: ToolbarProps) {
  const file = useRef<HTMLInputElement>(null);
  const bundle = useRef<HTMLInputElement>(null);
  useEffect(() => { bundle.current?.setAttribute("webkitdirectory", ""); }, []);
  const change = async (event: React.ChangeEvent<HTMLInputElement>) => {
    if (!event.target.files?.length) return;
    const result = await loadFromFiles(event.target.files);
    if (result.ok) onLoad(result.value); else onError(formatLoadError(result.error));
    event.target.value = "";
  };
  return (
    <header className="flex items-center justify-between border-b border-[var(--border)] bg-[var(--surface)] px-4 py-2">
      <div className="flex items-center gap-2"><strong>SysVista</strong>{projectName && <span className="text-sm text-[var(--muted)]">/ {projectName}</span>}</div>
      <div className="flex items-center gap-2">
        <button className="toolbar-button" onClick={() => file.current?.click()}><Upload className="h-4 w-4" /> Load JSON</button>
        <button className="toolbar-button" onClick={() => bundle.current?.click()}><FolderOpen className="h-4 w-4" /> Load bundle</button>
        <ThemeToggle theme={theme} onToggle={onToggleTheme} />
        <input ref={file} hidden type="file" accept=".json" onChange={change} />
        <input ref={bundle} hidden type="file" accept=".json" multiple onChange={change} />
      </div>
    </header>
  );
}
