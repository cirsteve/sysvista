import { useMemo, useEffect } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  type Node,
  type Edge,
  Handle,
  Position,
  type NodeProps,
  type NodeTypes,
  useReactFlow,
} from "@xyflow/react";
import dagre from "@dagrejs/dagre";
import type { Workflow, DetectedComponent, StepType } from "../types/schema";

const STEP_COLORS: Record<StepType, { border: string; bg: string; text: string; badge: string }> = {
  entry:    { border: "border-orange-500", bg: "bg-orange-950/80", text: "text-orange-100", badge: "bg-orange-500" },
  call:     { border: "border-green-500",  bg: "bg-green-950/80",  text: "text-green-100",  badge: "bg-green-500" },
  persist:  { border: "border-blue-500",   bg: "bg-blue-950/80",   text: "text-blue-100",   badge: "bg-blue-500" },
  dispatch: { border: "border-amber-500",  bg: "bg-amber-950/80",  text: "text-amber-100",  badge: "bg-amber-500" },
  response: { border: "border-cyan-500",   bg: "bg-cyan-950/80",   text: "text-cyan-100",   badge: "bg-cyan-500" },
};

const STEP_LABELS: Record<StepType, string> = {
  entry: "Entry",
  call: "Call",
  persist: "Persist",
  dispatch: "Dispatch",
  response: "Response",
};

interface WorkflowNodeData extends Record<string, unknown> {
  label: string;
  stepType: StepType;
  order: number;
  sublabel?: string;
}

function WorkflowStepNode({ data }: NodeProps) {
  const { label, stepType, order, sublabel } = data as unknown as WorkflowNodeData;
  const colors = STEP_COLORS[stepType];

  return (
    <div className={`border-2 ${colors.border} ${colors.bg} rounded-lg px-4 py-3 shadow-lg min-w-[160px]`}>
      <Handle type="target" position={Position.Left} className="!bg-gray-400" />
      <div className="flex items-center gap-2">
        <span className={`${colors.badge} text-white text-xs font-bold rounded-full w-5 h-5 flex items-center justify-center shrink-0`}>
          {order}
        </span>
        <div className="min-w-0">
          <div className={`text-sm font-semibold ${colors.text} truncate`}>{label}</div>
          <div className="text-xs text-gray-400">{STEP_LABELS[stepType]}{sublabel ? ` \u00b7 ${sublabel}` : ""}</div>
        </div>
      </div>
      <Handle type="source" position={Position.Right} className="!bg-gray-400" />
    </div>
  );
}

const nodeTypes: NodeTypes = {
  workflowStep: WorkflowStepNode,
};

interface WorkflowViewProps {
  workflow: Workflow;
  components: DetectedComponent[];
  onBack: () => void;
}

export function WorkflowView({ workflow, components, onBack }: WorkflowViewProps) {
  const compMap = new Map(components.map((c) => [c.id, c]));
  const { fitView } = useReactFlow();

  const { nodes, edges } = useMemo(() => {
    const g = new dagre.graphlib.Graph();
    g.setDefaultEdgeLabel(() => ({}));
    g.setGraph({ rankdir: "LR", ranksep: 120, nodesep: 40 });

    const nodes: Node<WorkflowNodeData>[] = workflow.steps.map((step) => {
      const comp = compMap.get(step.component_id);
      const nodeId = `wf-${step.order}`;
      g.setNode(nodeId, { width: 200, height: 60 });
      return {
        id: nodeId,
        type: "workflowStep",
        position: { x: 0, y: 0 },
        data: {
          label: comp?.name ?? step.component_id,
          stepType: step.step_type,
          order: step.order,
          sublabel: comp?.kind,
        },
      };
    });

    // Connect consecutive steps
    const edges: Edge[] = [];
    for (let i = 0; i < workflow.steps.length - 1; i++) {
      const fromId = `wf-${workflow.steps[i].order}`;
      const toId = `wf-${workflow.steps[i + 1].order}`;
      g.setEdge(fromId, toId);
      edges.push({
        id: `wfe-${i}`,
        source: fromId,
        target: toId,
        animated: true,
        style: { stroke: "#6b7280", strokeWidth: 1.5 },
      });
    }

    dagre.layout(g);

    for (const node of nodes) {
      const n = g.node(node.id);
      node.position = { x: n.x - 100, y: n.y - 30 };
    }

    return { nodes, edges };
  }, [workflow]);

  useEffect(() => {
    setTimeout(() => fitView({ padding: 0.3, duration: 300 }), 100);
  }, [nodes, fitView]);

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center gap-3 px-4 py-2 bg-gray-900/80 border-b border-gray-800">
        <button
          onClick={onBack}
          className="text-sm text-gray-400 hover:text-gray-200 transition-colors"
        >
          &larr; Back to Graph
        </button>
        <span className="text-gray-600">/</span>
        <span className="text-sm font-semibold text-gray-200">{workflow.name}</span>
        <span className="text-xs text-gray-500">{workflow.steps.length} steps</span>
      </div>
      <div className="flex-1">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          fitView
          minZoom={0.5}
          maxZoom={2}
          proOptions={{ hideAttribution: true }}
          nodesDraggable={false}
          nodesConnectable={false}
        >
          <Background color="#374151" gap={20} size={1} />
          <Controls position="bottom-left" />
        </ReactFlow>
      </div>
    </div>
  );
}
