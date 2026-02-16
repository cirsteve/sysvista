import { X, ChevronRight } from "lucide-react";
import type { Workflow, DetectedComponent, StepType } from "../types/schema";

const STEP_TYPE_CONFIG: Record<StepType, { label: string; color: string; bg: string }> = {
  entry:    { label: "Entry",    color: "text-orange-400", bg: "bg-orange-500/20" },
  call:     { label: "Call",     color: "text-green-400",  bg: "bg-green-500/20" },
  persist:  { label: "Persist",  color: "text-blue-400",   bg: "bg-blue-500/20" },
  dispatch: { label: "Dispatch", color: "text-amber-400",  bg: "bg-amber-500/20" },
  response: { label: "Response", color: "text-cyan-400",   bg: "bg-cyan-500/20" },
};

interface WorkflowPanelProps {
  workflows: Workflow[];
  selectedWorkflow: Workflow | null;
  components: DetectedComponent[];
  onSelectWorkflow: (workflow: Workflow | null) => void;
  onClose: () => void;
  onNavigateToComponent: (component: DetectedComponent) => void;
}

export function WorkflowPanel({
  workflows,
  selectedWorkflow,
  components,
  onSelectWorkflow,
  onClose,
  onNavigateToComponent,
}: WorkflowPanelProps) {
  const compMap = new Map(components.map((c) => [c.id, c]));

  // Sort by step count descending
  const sorted = [...workflows].sort((a, b) => b.steps.length - a.steps.length);

  return (
    <div className="absolute left-0 top-0 h-full w-80 bg-gray-900/95 backdrop-blur border-r border-gray-700 shadow-2xl overflow-y-auto z-50">
      <div className="p-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-bold text-gray-100">
            {selectedWorkflow ? "Workflow Trace" : "Workflows"}
          </h2>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-gray-700 text-gray-400 hover:text-gray-200"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {selectedWorkflow ? (
          <div>
            <button
              onClick={() => onSelectWorkflow(null)}
              className="text-xs text-gray-400 hover:text-gray-200 mb-3 flex items-center gap-1"
            >
              &larr; All workflows
            </button>
            <h3 className="text-sm font-semibold text-gray-200 mb-3">
              {selectedWorkflow.name}
            </h3>
            <div className="space-y-1">
              {selectedWorkflow.steps.map((step) => {
                const comp = compMap.get(step.component_id);
                const config = STEP_TYPE_CONFIG[step.step_type];
                return (
                  <button
                    key={`${step.order}-${step.component_id}`}
                    onClick={() => comp && onNavigateToComponent(comp)}
                    className="w-full text-left px-2 py-2 rounded hover:bg-gray-800 transition-colors flex items-center gap-2"
                  >
                    <span className={`text-xs font-bold ${config.color} ${config.bg} rounded px-1.5 py-0.5 shrink-0`}>
                      {step.order}
                    </span>
                    <div className="min-w-0 flex-1">
                      <div className="text-sm text-gray-300 truncate">
                        {comp?.name ?? step.component_id}
                      </div>
                      <div className={`text-xs ${config.color}`}>
                        {config.label}
                      </div>
                    </div>
                    <ChevronRight className="h-3.5 w-3.5 text-gray-600 shrink-0" />
                  </button>
                );
              })}
            </div>
          </div>
        ) : (
          <div className="space-y-1">
            {sorted.length === 0 ? (
              <p className="text-sm text-gray-500">No workflows detected</p>
            ) : (
              sorted.map((wf) => (
                <button
                  key={wf.id}
                  onClick={() => onSelectWorkflow(wf)}
                  className="w-full text-left px-3 py-2 rounded hover:bg-gray-800 transition-colors"
                >
                  <div className="text-sm text-gray-200 truncate">{wf.name}</div>
                  <div className="text-xs text-gray-500">{wf.steps.length} steps</div>
                </button>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}
