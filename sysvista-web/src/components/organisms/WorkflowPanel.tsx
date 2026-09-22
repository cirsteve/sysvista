import type { DetectedComponent } from "../../types/schema";
import type { Claim } from "../../types/v2";
import { PanelShell } from "../molecules/PanelShell";
import { ListItem } from "../molecules/ListItem";

interface WorkflowPanelProps {
  claims: Claim[];
  selectedClaim: Claim | null;
  components: DetectedComponent[];
  onSelectClaim: (claim: Claim | null) => void;
  onClose: () => void;
  onNavigateToComponent: (component: DetectedComponent) => void;
}

export function WorkflowPanel({
  claims,
  selectedClaim,
  components,
  onSelectClaim,
  onClose,
  onNavigateToComponent,
}: WorkflowPanelProps) {
  const componentById = new Map(components.map((component) => [component.id, component]));
  const sorted = [...claims].sort(
    (a, b) => b.object.entity_ids.length - a.object.entity_ids.length,
  );

  return (
    <PanelShell side="left" title={selectedClaim ? "Workflow Members" : "Workflows"} onClose={onClose}>
      {selectedClaim ? (
        <div>
          <button
            onClick={() => onSelectClaim(null)}
            className="text-xs text-gray-400 hover:text-gray-200 mb-3 flex items-center gap-1"
          >
            &larr; All workflows
          </button>
          <h3 className="text-sm font-semibold text-gray-200 mb-1">
            {selectedClaim.object.name}
          </h3>
          <p className="text-xs text-gray-500 mb-3">Members are unordered in legacy scan data.</p>
          <div className="space-y-1">
            {selectedClaim.object.entity_ids.map((entityId) => {
              const component = componentById.get(entityId);
              return (
                <ListItem
                  key={entityId}
                  label={component?.name ?? entityId}
                  kind={component?.kind}
                  onClick={() => component && onNavigateToComponent(component)}
                  showChevron={Boolean(component)}
                />
              );
            })}
          </div>
        </div>
      ) : (
        <div className="space-y-1">
          {sorted.length === 0 ? (
            <p className="text-sm text-gray-500">No workflows detected</p>
          ) : sorted.map((claim) => (
            <ListItem
              key={claim.id}
              label={claim.object.name}
              sublabel={`${claim.object.entity_ids.length} unordered members`}
              onClick={() => onSelectClaim(claim)}
            />
          ))}
        </div>
      )}
    </PanelShell>
  );
}
