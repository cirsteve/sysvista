import type { Workflow, DetectedComponent } from "../../types/schema";
import { STEP_TYPE_COLORS } from "../../lib/design-tokens";
import { PanelShell } from "../molecules/PanelShell";
import { ListItem } from "../molecules/ListItem";
import { Badge } from "../atoms/Badge";

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

  const title = selectedWorkflow ? "Workflow Trace" : "Workflows";

  return (
    <PanelShell side="left" title={title} onClose={onClose}>
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
              const config = STEP_TYPE_COLORS[step.step_type];
              return (
                <ListItem
                  key={`${step.order}-${step.component_id}`}
                  label={comp?.name ?? step.component_id}
                  sublabel={config.label}
                  onClick={() => comp && onNavigateToComponent(comp)}
                  showChevron
                  leading={
                    <Badge
                      label={String(step.order)}
                      variant="step"
                      stepType={step.step_type}
                    />
                  }
                />
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
              <ListItem
                key={wf.id}
                label={wf.name}
                sublabel={`${wf.steps.length} steps`}
                onClick={() => onSelectWorkflow(wf)}
              />
            ))
          )}
        </div>
      )}
    </PanelShell>
  );
}
