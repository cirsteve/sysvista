import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";
import { useGraphData } from "./hooks/useGraphData";
import { GraphCanvas } from "./components/GraphCanvas";
import { DetailPanel } from "./components/DetailPanel";
import { SearchBar } from "./components/SearchBar";
import { Toolbar } from "./components/Toolbar";
import { Legend } from "./components/Legend";
import { WorkflowPanel } from "./components/WorkflowPanel";
import { WorkflowView } from "./components/WorkflowView";
import { setupDropZone } from "./lib/loader";
import type { DetectedComponent } from "./types/schema";

function AppInner() {
  const {
    schema,
    nodes,
    edges,
    activeKinds,
    selectedNode,
    searchQuery,
    searchResults,
    connectedComponents,
    highlightedNodeIds,
    workflows,
    selectedWorkflow,
    viewMode,
    loadSchema,
    toggleKind,
    setSelectedNode,
    doSearch,
    selectWorkflow,
    setViewMode,
  } = useGraphData();

  const [focusNodeId, setFocusNodeId] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showWorkflowPanel, setShowWorkflowPanel] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const { fitView } = useReactFlow();

  // Auto-dismiss error after 4s
  useEffect(() => {
    if (!error) return;
    const t = setTimeout(() => setError(null), 4000);
    return () => clearTimeout(t);
  }, [error]);

  const handleLoad = useCallback(
    (data: Parameters<typeof loadSchema>[0]) => {
      try {
        loadSchema(data);
        setShowWorkflowPanel(false);
      } catch (err) {
        console.error("Graph build failed:", err);
        setError(
          err instanceof Error ? err.message : "Failed to build graph",
        );
      }
    },
    [loadSchema],
  );

  // Set up drag-and-drop
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    return setupDropZone(el, handleLoad, setError, setIsDragging);
  }, [handleLoad]);

  const handleNodeClick = useCallback(
    (component: DetectedComponent) => {
      setSelectedNode(component);
      setFocusNodeId(null);
    },
    [setSelectedNode],
  );

  const handleNavigate = useCallback(
    (component: DetectedComponent) => {
      setSelectedNode(component);
      setFocusNodeId(component.id);
      // Switch to graph view when navigating to a component from workflow panel
      if (viewMode === "workflow") {
        selectWorkflow(null);
      }
    },
    [setSelectedNode, viewMode, selectWorkflow],
  );

  const handleSearchSelect = useCallback(
    (component: DetectedComponent) => {
      setSelectedNode(component);
      setFocusNodeId(component.id);
    },
    [setSelectedNode],
  );

  // For large graphs, only show edges connected to the selected node
  const visibleEdges = useMemo(() => {
    if (edges.length <= 500) return edges;
    if (!selectedNode) return [];
    return edges.filter(
      (e) => e.source === selectedNode.id || e.target === selectedNode.id,
    );
  }, [edges, selectedNode]);

  const handleFitView = useCallback(() => {
    fitView({ padding: 0.2, duration: 300 });
  }, [fitView]);

  const handleToggleWorkflows = useCallback(() => {
    setShowWorkflowPanel((prev) => !prev);
  }, []);

  const handleBackToGraph = useCallback(() => {
    selectWorkflow(null);
  }, [selectWorkflow]);

  return (
    <div ref={containerRef} className="flex flex-col h-screen">
      <Toolbar
        projectName={schema?.project_name}
        stats={
          schema
            ? {
                components: schema.components.length,
                edges: schema.edges.length,
                files: schema.scan_stats.files_scanned,
              }
            : undefined
        }
        workflowCount={workflows.length}
        onLoad={handleLoad}
        onError={setError}
        onFitView={handleFitView}
        onToggleWorkflows={handleToggleWorkflows}
      />

      <div className="flex-1 relative">
        {/* Search bar overlay (only in graph view) */}
        {viewMode === "graph" && (
          <div className="absolute top-3 left-3 z-40">
            <SearchBar
              query={searchQuery}
              results={searchResults}
              activeKinds={activeKinds}
              onSearch={doSearch}
              onSelect={handleSearchSelect}
              onToggleKind={toggleKind}
            />
          </div>
        )}

        {schema ? (
          <>
            {viewMode === "workflow" && selectedWorkflow ? (
              <>
                <WorkflowView
                  workflow={selectedWorkflow}
                  components={schema.components}
                  onBack={handleBackToGraph}
                />
                <Legend mode="workflow" />
              </>
            ) : (
              <>
                <GraphCanvas
                  nodes={nodes}
                  edges={visibleEdges}
                  onNodeClick={handleNodeClick}
                  focusNodeId={focusNodeId}
                  highlightedNodeIds={highlightedNodeIds}
                />
                <Legend mode="graph" />
              </>
            )}
          </>
        ) : (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="text-6xl mb-4 opacity-20">&#9678;</div>
              <h2 className="text-xl font-semibold text-gray-400 mb-2">
                No architecture loaded
              </h2>
              <p className="text-sm text-gray-500 mb-4">
                Load a SysVista JSON file or drag and drop one here
              </p>
              <p className="text-xs text-gray-600">
                Generate one with:{" "}
                <code className="bg-gray-800 px-1.5 py-0.5 rounded">
                  sysvista-cli scan /path/to/project -o output.json
                </code>
              </p>
            </div>
          </div>
        )}

        {/* Workflow panel */}
        {showWorkflowPanel && schema && (
          <WorkflowPanel
            workflows={workflows}
            selectedWorkflow={selectedWorkflow}
            components={schema.components}
            onSelectWorkflow={selectWorkflow}
            onClose={() => setShowWorkflowPanel(false)}
            onNavigateToComponent={handleNavigate}
          />
        )}

        {/* Detail panel */}
        {selectedNode && viewMode === "graph" && (
          <DetailPanel
            component={selectedNode}
            connectedComponents={connectedComponents}
            onClose={() => setSelectedNode(null)}
            onNavigate={handleNavigate}
          />
        )}

        {/* Drag overlay */}
        {isDragging && (
          <div className="absolute inset-0 z-50 flex items-center justify-center bg-gray-950/60 border-2 border-dashed border-blue-500 rounded-lg m-2 pointer-events-none">
            <div className="text-center">
              <div className="text-4xl mb-2 opacity-60">&#8615;</div>
              <p className="text-sm text-blue-400 font-medium">Drop JSON file to load</p>
            </div>
          </div>
        )}
      </div>

      {/* Error toast */}
      {error && (
        <div className="fixed bottom-4 left-1/2 -translate-x-1/2 z-50 bg-red-900/90 text-red-200 px-4 py-2.5 rounded-lg border border-red-700 text-sm shadow-lg">
          {error}
        </div>
      )}
    </div>
  );
}

export default function App() {
  return (
    <ReactFlowProvider>
      <AppInner />
    </ReactFlowProvider>
  );
}
