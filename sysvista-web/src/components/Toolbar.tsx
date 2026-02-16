import { Upload, Maximize, GitBranch } from "lucide-react";
import { useRef } from "react";
import type { SysVistaOutput } from "../types/schema";
import { loadFromFile } from "../lib/loader";

interface ToolbarProps {
  projectName?: string;
  stats?: { components: number; edges: number; files: number };
  workflowCount?: number;
  onLoad: (data: SysVistaOutput) => void;
  onError: (message: string) => void;
  onFitView: () => void;
  onToggleWorkflows?: () => void;
}

export function Toolbar({ projectName, stats, workflowCount, onLoad, onError, onFitView, onToggleWorkflows }: ToolbarProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    try {
      const data = await loadFromFile(file);
      onLoad(data);
    } catch (err) {
      onError(err instanceof Error ? err.message : "Failed to load file");
    }
    // Reset so the same file can be re-selected
    e.target.value = "";
  };

  return (
    <div className="flex items-center justify-between px-4 py-2 bg-gray-900 border-b border-gray-800">
      <div className="flex items-center gap-3">
        <h1 className="text-base font-bold text-gray-100 tracking-tight">
          SysVista
        </h1>
        {projectName && (
          <>
            <span className="text-gray-600">/</span>
            <span className="text-sm text-gray-400">{projectName}</span>
          </>
        )}
        {stats && (
          <span className="text-xs text-gray-500">
            {stats.components} components, {stats.edges} edges, {stats.files}{" "}
            files
          </span>
        )}
      </div>

      <div className="flex items-center gap-2">
        {onToggleWorkflows && workflowCount !== undefined && workflowCount > 0 && (
          <button
            onClick={onToggleWorkflows}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg border border-gray-700 transition-colors"
          >
            <GitBranch className="h-3.5 w-3.5" />
            Workflows
            <span className="bg-gray-700 text-gray-300 text-xs rounded-full px-1.5 py-0.5 min-w-[1.25rem] text-center">
              {workflowCount}
            </span>
          </button>
        )}
        <button
          onClick={() => fileInputRef.current?.click()}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg border border-gray-700 transition-colors"
        >
          <Upload className="h-3.5 w-3.5" />
          Load JSON
        </button>
        <button
          onClick={onFitView}
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg border border-gray-700 transition-colors"
        >
          <Maximize className="h-3.5 w-3.5" />
          Fit
        </button>
        <input
          ref={fileInputRef}
          type="file"
          accept=".json"
          onChange={handleFileChange}
          className="hidden"
        />
      </div>
    </div>
  );
}
