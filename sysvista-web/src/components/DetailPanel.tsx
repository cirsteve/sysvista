import { X, FileCode, MapPin, Tag, Link } from "lucide-react";
import type { DetectedComponent } from "../types/schema";

const KIND_COLORS: Record<string, string> = {
  model: "text-blue-400 bg-blue-500/10 border-blue-500/30",
  service: "text-green-400 bg-green-500/10 border-green-500/30",
  transport: "text-orange-400 bg-orange-500/10 border-orange-500/30",
  transform: "text-purple-400 bg-purple-500/10 border-purple-500/30",
};

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
  const kindClass = KIND_COLORS[component.kind] ?? "";

  return (
    <div className="absolute right-0 top-0 h-full w-80 bg-gray-900/95 backdrop-blur border-l border-gray-700 shadow-2xl overflow-y-auto z-50">
      <div className="p-4">
        {/* Header */}
        <div className="flex items-start justify-between mb-4">
          <div className="flex-1 min-w-0">
            <h2 className="text-lg font-bold text-gray-100 truncate">
              {component.name}
            </h2>
            <span
              className={`inline-block mt-1 px-2 py-0.5 text-xs font-medium rounded border ${kindClass}`}
            >
              {component.kind}
            </span>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-gray-700 text-gray-400 hover:text-gray-200 ml-2"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Source */}
        <div className="mb-4">
          <div className="flex items-center gap-1.5 text-xs font-medium text-gray-400 mb-1">
            <FileCode className="h-3.5 w-3.5" />
            Source
          </div>
          <div className="text-sm text-gray-300 font-mono bg-gray-800 rounded px-2 py-1">
            {component.source.file}
            {component.source.line_start && (
              <span className="text-gray-500">
                :{component.source.line_start}
              </span>
            )}
          </div>
        </div>

        {/* Language */}
        <div className="mb-4">
          <div className="flex items-center gap-1.5 text-xs font-medium text-gray-400 mb-1">
            <Tag className="h-3.5 w-3.5" />
            Language
          </div>
          <div className="text-sm text-gray-300">{component.language}</div>
        </div>

        {/* Transport details */}
        {component.transport_protocol && (
          <div className="mb-4">
            <div className="flex items-center gap-1.5 text-xs font-medium text-gray-400 mb-1">
              <MapPin className="h-3.5 w-3.5" />
              Transport
            </div>
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
                <span
                  key={field}
                  className="text-xs font-mono bg-gray-800 text-gray-300 rounded px-1.5 py-0.5"
                >
                  {field}
                </span>
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
            <div className="flex items-center gap-1.5 text-xs font-medium text-gray-400 mb-2">
              <Link className="h-3.5 w-3.5" />
              Connected ({connectedComponents.length})
            </div>
            <div className="space-y-1">
              {connectedComponents.map((conn) => (
                <button
                  key={conn.id}
                  onClick={() => onNavigate(conn)}
                  className="w-full text-left px-2 py-1.5 rounded text-sm hover:bg-gray-800 transition-colors"
                >
                  <span
                    className={`inline-block w-2 h-2 rounded-full mr-2 ${
                      conn.kind === "model"
                        ? "bg-blue-400"
                        : conn.kind === "service"
                          ? "bg-green-400"
                          : conn.kind === "transport"
                            ? "bg-orange-400"
                            : "bg-purple-400"
                    }`}
                  />
                  <span className="text-gray-300">{conn.name}</span>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
