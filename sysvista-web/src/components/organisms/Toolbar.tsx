import type { LoadedSnapshot } from "../../lib/loader";
import { ThemeToggle } from "../atoms/ThemeToggle";
import { ImportDialog } from "./ImportDialog";

interface ToolbarProps {
  projectName?: string;
  theme: "light" | "dark";
  onToggleTheme: () => void;
  onLoad: (data: LoadedSnapshot) => void;
  onError: (message: string) => void;
  lens: "structure" | "flow";
  flowHops: number;
  onLensChange: (lens: "structure" | "flow") => void;
  onFlowHopsChange: (hops: number) => void;
}

export function Toolbar({ projectName, theme, onToggleTheme, onLoad, onError, lens, flowHops, onLensChange, onFlowHopsChange }: ToolbarProps) {
  return (
    <header className="flex items-center justify-between border-b border-[var(--border)] bg-[var(--surface)] px-4 py-2">
      <div className="flex items-center gap-2"><strong>SysVista</strong>{projectName && <span className="text-sm text-[var(--muted)]">/ {projectName}</span>}</div>
      <div className="flex items-center gap-2">
        <ImportDialog onLoad={onLoad} onError={onError} />
        <div className="flex rounded border border-[var(--border)]" aria-label="Diagram lens">
          <button type="button" className={`px-2 py-1 text-xs ${lens === "structure" ? "bg-sky-600 text-white" : ""}`} onClick={() => onLensChange("structure")}>Structure</button>
          <button type="button" className={`px-2 py-1 text-xs ${lens === "flow" ? "bg-sky-600 text-white" : ""}`} onClick={() => onLensChange("flow")}>Flow</button>
        </div>
        {lens === "flow" && <label className="flex items-center gap-1 text-xs">Hops <input aria-label="Flow hops" className="w-12 rounded border border-[var(--border)] bg-transparent px-1 py-1" type="number" min={0} max={8} value={flowHops} onChange={(event) => onFlowHopsChange(Number(event.target.value))} /></label>}
        <ThemeToggle theme={theme} onToggle={onToggleTheme} />
      </div>
    </header>
  );
}
