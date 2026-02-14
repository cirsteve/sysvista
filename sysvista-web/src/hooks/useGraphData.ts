import { useState, useCallback, useMemo } from "react";
import type { Node, Edge } from "@xyflow/react";
import type {
  SysVistaOutput,
  DetectedComponent,
  ComponentKind,
} from "../types/schema";
import { buildGraph, type GraphNode } from "../lib/graph-adapter";
import { initSearch, search } from "../lib/search";

const ALL_KINDS: ComponentKind[] = ["model", "service", "transport", "transform"];

const FLOW_LABELS = new Set(["handles", "persists", "transforms", "consumes", "produces"]);

export function useGraphData() {
  const [schema, setSchema] = useState<SysVistaOutput | null>(null);
  const [activeKinds, setActiveKinds] = useState<Set<ComponentKind>>(
    new Set(ALL_KINDS),
  );
  const [selectedNode, setSelectedNode] = useState<DetectedComponent | null>(
    null,
  );
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<DetectedComponent[]>([]);

  const loadSchema = useCallback((data: SysVistaOutput) => {
    setSchema(data);
    initSearch(data.components);
    setSelectedNode(null);
    setSearchQuery("");
    setSearchResults([]);
  }, []);

  const toggleKind = useCallback((kind: ComponentKind) => {
    setActiveKinds((prev) => {
      const next = new Set(prev);
      if (next.has(kind)) {
        next.delete(kind);
      } else {
        next.add(kind);
      }
      return next;
    });
  }, []);

  const doSearch = useCallback((query: string) => {
    setSearchQuery(query);
    setSearchResults(search(query));
  }, []);

  const { nodes, edges } = useMemo((): {
    nodes: Node[];
    edges: Edge[];
  } => {
    if (!schema) return { nodes: [], edges: [] };
    try {
      return buildGraph(schema, activeKinds);
    } catch (err) {
      console.error("buildGraph failed:", err);
      return { nodes: [], edges: [] };
    }
  }, [schema, activeKinds]);

  const connectedComponents = useMemo(() => {
    if (!schema || !selectedNode) return [];
    const connectedIds = new Set<string>();
    for (const edge of schema.edges) {
      if (edge.from_id === selectedNode.id) connectedIds.add(edge.to_id);
      if (edge.to_id === selectedNode.id) connectedIds.add(edge.from_id);
    }
    return schema.components.filter((c) => connectedIds.has(c.id));
  }, [schema, selectedNode]);

  /**
   * Trace workflow from a component by following flow edges (handles, persists, transforms).
   * BFS in both directions to find the full flow chain.
   */
  const traceWorkflow = useCallback(
    (componentId: string): { nodeIds: Set<string>; edgeIds: Set<string> } => {
      if (!schema) return { nodeIds: new Set(), edgeIds: new Set() };

      // Build adjacency from flow edges only
      const flowEdges = schema.edges.filter(
        (e) => e.label && FLOW_LABELS.has(e.label),
      );

      const nodeIds = new Set<string>();
      const edgeIds = new Set<string>();
      const queue = [componentId];
      nodeIds.add(componentId);

      while (queue.length > 0) {
        const current = queue.shift()!;
        for (let i = 0; i < flowEdges.length; i++) {
          const e = flowEdges[i];
          let neighbor: string | null = null;
          if (e.from_id === current) neighbor = e.to_id;
          else if (e.to_id === current) neighbor = e.from_id;

          if (neighbor && !nodeIds.has(neighbor)) {
            nodeIds.add(neighbor);
            edgeIds.add(`flow-${i}`);
            queue.push(neighbor);
          } else if (neighbor) {
            // Edge still part of the trace even if node already visited
            edgeIds.add(`flow-${i}`);
          }
        }
      }

      return { nodeIds, edgeIds };
    },
    [schema],
  );

  // When a transport is selected, auto-trace its workflow
  const highlightedNodeIds = useMemo((): Set<string> | null => {
    if (!selectedNode || selectedNode.kind !== "transport") return null;
    const { nodeIds } = traceWorkflow(selectedNode.id);
    // Only highlight if there are flow connections (more than just the selected node)
    return nodeIds.size > 1 ? nodeIds : null;
  }, [selectedNode, traceWorkflow]);

  return {
    schema,
    nodes,
    edges,
    activeKinds,
    selectedNode,
    searchQuery,
    searchResults,
    connectedComponents,
    highlightedNodeIds,
    loadSchema,
    toggleKind,
    setSelectedNode,
    doSearch,
  };
}
