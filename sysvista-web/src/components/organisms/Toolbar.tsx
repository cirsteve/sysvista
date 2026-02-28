import { Upload, Maximize, GitBranch, Workflow } from "lucide-react";
import { useRef } from "react";
import type { SysVistaOutput } from "../../types/schema";
import type { ViewMode } from "../../hooks/useGraphData";
import { loadFromFile } from "../../lib/loader";
import { IconButton } from "../atoms/IconButton";

interface ToolbarProps {
  projectName?: string;
  stats?: { components: number; edges: number; files: number };
  viewMode: ViewMode;
  flowEdgeCount: number;
  workflowCount: number;
  onLoad: (data: SysVistaOutput) => void;
  onError: (message: string) => void;
  onFitView: () => void;
  onToggleFlowView: () => void;
  onToggleWorkflows?: () => void;
}

export function Toolbar({ projectName, stats, viewMode, flowEdgeCount, workflowCount, onLoad, onError, onFitView, onToggleFlowView, onToggleWorkflows }: ToolbarProps) {
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

  const isFlowActive = viewMode === "flow";

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
        {flowEdgeCount > 0 && (
          <IconButton
            icon={Workflow}
            label="Flow View"
            badge={flowEdgeCount}
            onClick={onToggleFlowView}
            variant={isFlowActive ? "active" : "default"}
            badgeColorClass={isFlowActive ? "bg-cyan-800 text-cyan-200" : undefined}
          />
        )}
        {isFlowActive && onToggleWorkflows && workflowCount > 0 && (
          <IconButton
            icon={GitBranch}
            label="Workflows"
            badge={workflowCount}
            onClick={onToggleWorkflows}
          />
        )}
        <IconButton
          icon={Upload}
          label="Load JSON"
          onClick={() => fileInputRef.current?.click()}
        />
        <IconButton
          icon={Maximize}
          label="Fit"
          onClick={onFitView}
        />
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
