import { FileCode, MapPin, Tag, Link } from "lucide-react";
import type { DetectedComponent } from "../../types/schema";
import { PanelShell } from "../molecules/PanelShell";
import { FieldGroup } from "../molecules/FieldGroup";
import { ListItem } from "../molecules/ListItem";
import { Badge } from "../atoms/Badge";
import { SectionHeader } from "../atoms/SectionHeader";

interface DetailPanelProps {
  component: DetectedComponent;
  connectedComponents: DetectedComponent[];
  onClose: () => void;
  onNavigate: (component: DetectedComponent) => void;
}

export function DetailPanel({
  component,
  connectedComponents,
  onClose,
  onNavigate,
}: DetailPanelProps) {
  return (
    <PanelShell side="right" title="" onClose={onClose}>
      {/* Header */}
      <div className="-mt-4 mb-4">
        <h2 className="text-lg font-bold text-gray-100 truncate">
          {component.name}
        </h2>
        <div className="mt-1">
          <Badge label={component.kind} variant="kind" kind={component.kind} />
        </div>
      </div>

      {/* Source */}
      <FieldGroup label="Source" icon={FileCode}>
        <div className="text-sm text-gray-300 font-mono bg-gray-800 rounded px-2 py-1">
          {component.source.file}
          {component.source.line_start && (
            <span className="text-gray-500">
              :{component.source.line_start}
            </span>
          )}
        </div>
      </FieldGroup>

      {/* Language */}
      <FieldGroup label="Language" icon={Tag}>
        <div className="text-sm text-gray-300">{component.language}</div>
      </FieldGroup>

      {/* Transport details */}
      {component.transport_protocol && (
        <FieldGroup label="Transport" icon={MapPin}>
          <div className="text-sm text-gray-300">
            {component.http_method && (
              <span className="font-mono font-bold mr-1">
                {component.http_method}
              </span>
            )}
            {component.http_path && (
              <span className="font-mono">{component.http_path}</span>
            )}
            {!component.http_path && component.transport_protocol}
          </div>
        </FieldGroup>
      )}

      {/* Payload Flow */}
      {(component.consumes || component.produces) && (
        <div className="mb-4">
          <div className="text-xs font-medium text-gray-400 mb-1">
            Payload Flow
          </div>
          {component.consumes && component.consumes.length > 0 && (
            <div className="mb-1.5">
              <span className="text-xs text-gray-500 mr-1.5">Accepts:</span>
              <div className="inline-flex flex-wrap gap-1">
                {component.consumes.map((t) => (
                  <Badge
                    key={t}
                    label={t}
                    variant="type"
                    colorClass="bg-pink-500/10 text-pink-300 border-pink-500/30"
                  />
                ))}
              </div>
            </div>
          )}
          {component.produces && component.produces.length > 0 && (
            <div>
              <span className="text-xs text-gray-500 mr-1.5">Returns:</span>
              <div className="inline-flex flex-wrap gap-1">
                {component.produces.map((t) => (
                  <Badge
                    key={t}
                    label={t}
                    variant="type"
                    colorClass="bg-cyan-500/10 text-cyan-300 border-cyan-500/30"
                  />
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Model fields */}
      {component.model_fields && component.model_fields.length > 0 && (
        <div className="mb-4">
          <div className="text-xs font-medium text-gray-400 mb-1">
            Fields
          </div>
          <div className="flex flex-wrap gap-1">
            {component.model_fields.map((field) => (
              <Badge key={field} label={field} variant="field" />
            ))}
          </div>
        </div>
      )}

      {/* Metadata */}
      {Object.keys(component.metadata).length > 0 && (
        <div className="mb-4">
          <div className="text-xs font-medium text-gray-400 mb-1">
            Metadata
          </div>
          {Object.entries(component.metadata).map(([key, value]) => (
            <div key={key} className="text-sm text-gray-300">
              <span className="text-gray-500">{key}:</span> {value}
            </div>
          ))}
        </div>
      )}

      {/* Connected components */}
      {connectedComponents.length > 0 && (
        <div>
          <SectionHeader icon={Link} label="Connected" count={connectedComponents.length} />
          <div className="space-y-1 mt-1">
            {connectedComponents.map((conn) => (
              <ListItem
                key={conn.id}
                kind={conn.kind}
                label={conn.name}
                onClick={() => onNavigate(conn)}
              />
            ))}
          </div>
        </div>
      )}
    </PanelShell>
  );
}
