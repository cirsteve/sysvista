import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  type Node,
  type Edge,
  type NodeTypes,
  useReactFlow,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { useCallback, useEffect, useMemo } from "react";
import type { DetectedComponent } from "../types/schema";
import type { GraphNode } from "../lib/graph-adapter";
import { ModelNode } from "./nodes/ModelNode";
import { ServiceNode } from "./nodes/ServiceNode";
import { TransportNode } from "./nodes/TransportNode";
import { TransformNode } from "./nodes/TransformNode";
import { GroupLabelNode } from "./nodes/GroupLabelNode";
import { ClusterLabelNode } from "./nodes/ClusterLabelNode";

const nodeTypes: NodeTypes = {
  model: ModelNode,
  service: ServiceNode,
  transport: TransportNode,
  transform: TransformNode,
  groupLabel: GroupLabelNode,
  clusterLabel: ClusterLabelNode,
};

interface GraphCanvasProps {
  nodes: Node[];
  edges: Edge[];
  onNodeClick: (component: DetectedComponent) => void;
  focusNodeId?: string | null;
  highlightedNodeIds?: Set<string> | null;
}

function GraphCanvasInner({
  nodes,
  edges,
  onNodeClick,
  focusNodeId,
  highlightedNodeIds,
}: GraphCanvasProps) {
  const { fitView, setCenter } = useReactFlow();

  const handleNodeClick = useCallback(
    (_: React.MouseEvent, node: Node) => {
      // Skip label nodes
      if (node.type === "groupLabel" || node.type === "clusterLabel") return;
      const data = node.data as Record<string, unknown>;
      if (data.component) {
        onNodeClick(data.component as DetectedComponent);
      }
    },
    [onNodeClick],
  );

  // Apply highlight state to nodes
  const styledNodes = useMemo(() => {
    if (!highlightedNodeIds || highlightedNodeIds.size === 0) return nodes;
    return nodes.map((node) => {
      if (node.type === "groupLabel" || node.type === "clusterLabel") return node;
      const isHighlighted = highlightedNodeIds.has(node.id);
      return {
        ...node,
        data: { ...node.data, highlighted: isHighlighted },
        style: isHighlighted ? undefined : { opacity: 0.3 },
      };
    });
  }, [nodes, highlightedNodeIds]);

  useEffect(() => {
    if (focusNodeId) {
      const node = nodes.find((n) => n.id === focusNodeId);
      if (node) {
        setCenter(node.position.x + 90, node.position.y + 30, {
          zoom: 1.5,
          duration: 500,
        });
      }
    }
  }, [focusNodeId, nodes, setCenter]);

  useEffect(() => {
    if (nodes.length > 0 && !focusNodeId) {
      setTimeout(() => fitView({ padding: 0.2, duration: 300 }), 100);
    }
  }, [nodes, fitView, focusNodeId]);

  return (
    <ReactFlow
      nodes={styledNodes}
      edges={edges}
      nodeTypes={nodeTypes}
      onNodeClick={handleNodeClick}
      fitView
      minZoom={0.1}
      maxZoom={3}
      proOptions={{ hideAttribution: true }}
    >
      <Background color="#374151" gap={20} size={1} />
      <Controls position="bottom-left" />
      <MiniMap
        nodeColor={(node) => {
          // Amber for hub nodes
          const data = node.data as GraphNode | undefined;
          if (data?.hubTier === "high") return "#f59e0b";
          if (data?.hubTier === "medium") return "#d97706";

          const colors: Record<string, string> = {
            model: "#3b82f6",
            service: "#22c55e",
            transport: "#f97316",
            transform: "#a855f7",
          };
          return colors[node.type ?? ""] ?? "#6b7280";
        }}
        maskColor="rgba(0,0,0,0.7)"
        position="bottom-right"
      />
    </ReactFlow>
  );
}

export function GraphCanvas(props: GraphCanvasProps) {
  return <GraphCanvasInner {...props} />;
}
