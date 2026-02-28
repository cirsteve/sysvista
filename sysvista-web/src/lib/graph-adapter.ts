import dagre from "@dagrejs/dagre";
import type { Node, Edge } from "@xyflow/react";
import type { SysVistaOutput, DetectedComponent, DetectedEdge, ComponentKind } from "../types/schema";
import { classifyComponents, detectHubs, type HubInfo } from "./clustering";
import { KIND_NODE_SIZE } from "./design-tokens";
import type { ClusterLabelData } from "../components/nodes/ClusterLabelNode";

export interface GraphNode extends Record<string, unknown> {
  component: DetectedComponent;
  hubTier: HubInfo["tier"];
  degree: number;
  cluster: string;
  highlighted?: boolean;
  direction?: "TB" | "LR";
}

export interface GroupLabelNode extends Record<string, unknown> {
  label: string;
  count: number;
  kind: ComponentKind;
}

const KIND_CONFIG = KIND_NODE_SIZE;

export const FLOW_LABELS = new Set(["handles", "persists", "transforms", "consumes", "produces", "calls", "dispatches"]);
const PAYLOAD_LABELS = new Set(["consumes", "produces"]);

// Dagre can't handle dense graphs — fall back to cluster grid layout above this threshold
const MAX_DAGRE_EDGES = 2000;

// --- Shared helpers ---

interface MergedEdge {
  from_id: string;
  to_id: string;
  labels: string[];
  payload_types: string[];
}

const deduplicateEdges = (edges: DetectedEdge[]): MergedEdge[] =>
  [...edges.reduce((acc, e) => {
    const key = `${e.from_id}->${e.to_id}`;
    const existing = acc.get(key);
    if (existing) {
      if (e.label && !existing.labels.includes(e.label)) existing.labels.push(e.label);
      if (e.payload_type && !existing.payload_types.includes(e.payload_type)) existing.payload_types.push(e.payload_type);
    } else {
      acc.set(key, {
        from_id: e.from_id,
        to_id: e.to_id,
        labels: e.label ? [e.label] : [],
        payload_types: e.payload_type ? [e.payload_type] : [],
      });
    }
    return acc;
  }, new Map<string, MergedEdge>()).values()];

const toComponentNode = (
  comp: DetectedComponent,
  position: { x: number; y: number },
  hubMap: Map<string, HubInfo>,
  clusterMap: Map<string, string>,
  direction?: "TB" | "LR",
): Node<GraphNode> => {
  const hub = hubMap.get(comp.id) ?? { tier: "normal" as const, degree: 0 };
  return {
    id: comp.id,
    type: comp.kind,
    position,
    data: {
      component: comp,
      hubTier: hub.tier,
      degree: hub.degree,
      cluster: clusterMap.get(comp.id) ?? "Other",
      ...(direction && { direction }),
    },
  };
};

const classifyEdge = (labels: string[]) => ({
  isPayload: labels.some((l) => PAYLOAD_LABELS.has(l)),
  isFlow: labels.some((l) => FLOW_LABELS.has(l)),
  isCalls: labels.includes("calls"),
  isDispatches: labels.includes("dispatches"),
});

const edgeStroke = ({ isPayload, isCalls, isDispatches, isFlow }: ReturnType<typeof classifyEdge>) =>
  isPayload ? { stroke: "#f472b6", labelFill: "#f9a8d4" }
  : isCalls ? { stroke: "#22c55e", labelFill: "#86efac" }
  : isDispatches ? { stroke: "#f59e0b", labelFill: "#fcd34d" }
  : isFlow ? { stroke: "#06b6d4", labelFill: "#67e8f9" }
  : { stroke: "#6b7280", labelFill: "#9ca3af" };

const formatEdgeLabel = (e: MergedEdge): string | undefined => {
  const base = e.labels.join(", ");
  if (!base) return undefined;
  return e.payload_types.length > 0 ? `${base} [${e.payload_types.join(", ")}]` : base;
};

// --- Layout ---

function clusterGridLayout(
  components: DetectedComponent[],
  clusterMap: Map<string, string>,
  hubMap: Map<string, HubInfo>,
): { positions: Map<string, { x: number; y: number }>; headerNodes: Node<ClusterLabelData>[] } {
  const cols = Math.max(4, Math.ceil(Math.sqrt(components.length / 2)));
  const cellW = 240;
  const cellH = 100;
  const headerH = 50;
  const groupGap = 60;

  // Group by cluster
  const clusters = new Map<string, DetectedComponent[]>();
  for (const c of components) {
    const cluster = clusterMap.get(c.id) ?? "Other";
    let bucket = clusters.get(cluster);
    if (!bucket) {
      bucket = [];
      clusters.set(cluster, bucket);
    }
    bucket.push(c);
  }

  // Sort clusters: largest first, "Other" last; sort members by degree desc
  const sortedClusters = [...clusters.entries()]
    .sort((a, b) => {
      if (a[0] === "Other") return 1;
      if (b[0] === "Other") return -1;
      return b[1].length - a[1].length;
    })
    .map(([name, comps]) => [name, [...comps].sort((a, b) =>
      (hubMap.get(b.id)?.degree ?? 0) - (hubMap.get(a.id)?.degree ?? 0),
    )] as [string, DetectedComponent[]]);

  // Lay out each cluster sequentially, accumulating Y offset.
  // Mutate accumulators in place — this runs on dense graphs (>2000 edges)
  // where repeated Map/Array spreading would be expensive.
  const positions = new Map<string, { x: number; y: number }>();
  const headerNodes: Node<ClusterLabelData>[] = [];
  let offsetY = 0;

  for (const [clusterName, comps] of sortedClusters) {
    if (comps.length === 0) continue;
    const rows = Math.ceil(comps.length / cols);

    headerNodes.push({
      id: `cluster-${clusterName}`,
      type: "clusterLabel",
      position: { x: 0, y: offsetY },
      data: { label: clusterName, count: comps.length },
      selectable: false,
      draggable: false,
    });

    for (let i = 0; i < comps.length; i++) {
      positions.set(comps[i].id, {
        x: (i % cols) * cellW,
        y: offsetY + headerH + Math.floor(i / cols) * cellH,
      });
    }

    offsetY += headerH + rows * cellH + groupGap;
  }

  return { positions, headerNodes };
}

function dagreLayout(
  components: DetectedComponent[],
  edges: MergedEdge[],
  rankdir: "TB" | "LR",
  nodesep: number,
  ranksep: number,
): Map<string, { x: number; y: number }> {
  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir, nodesep, ranksep });

  components.forEach((comp) => {
    const config = KIND_CONFIG[comp.kind];
    g.setNode(comp.id, { width: config.width, height: config.height });
  });

  edges.forEach((edge) => g.setEdge(edge.from_id, edge.to_id));

  dagre.layout(g);

  return new Map(components.map((comp) => {
    const n = g.node(comp.id);
    const config = KIND_CONFIG[comp.kind];
    return [comp.id, { x: n.x - config.width / 2, y: n.y - config.height / 2 }];
  }));
}

// --- Graph builders ---

export function buildGraph(
  data: SysVistaOutput,
  activeKinds: Set<ComponentKind>,
): { nodes: Node[]; edges: Edge[] } {
  const filteredComponents = data.components.filter((c) => activeKinds.has(c.kind));
  const visibleIds = new Set(filteredComponents.map((c) => c.id));
  const filteredEdges = data.edges.filter((e) => visibleIds.has(e.from_id) && visibleIds.has(e.to_id));
  const uniqueEdges = deduplicateEdges(filteredEdges);

  const clusterMap = classifyComponents(filteredComponents);
  const hubMap = detectHubs(filteredComponents, filteredEdges);

  let componentNodes: Node[];

  if (uniqueEdges.length > MAX_DAGRE_EDGES) {
    const { positions, headerNodes } = clusterGridLayout(filteredComponents, clusterMap, hubMap);
    componentNodes = [
      ...headerNodes,
      ...filteredComponents.map((comp) =>
        toComponentNode(comp, positions.get(comp.id) ?? { x: 0, y: 0 }, hubMap, clusterMap),
      ),
    ];
  } else {
    const positions = dagreLayout(filteredComponents, uniqueEdges, "TB", 60, 80);
    componentNodes = filteredComponents.map((comp) =>
      toComponentNode(comp, positions.get(comp.id) ?? { x: 0, y: 0 }, hubMap, clusterMap),
    );
  }

  const edges: Edge[] = uniqueEdges
    .map((e, i) => {
      const cls = classifyEdge(e.labels);
      const { stroke, labelFill } = edgeStroke(cls);
      return {
        id: `e-${i}`,
        source: e.from_id,
        target: e.to_id,
        label: formatEdgeLabel(e),
        animated: cls.isFlow,
        zIndex: cls.isPayload ? 10 : (cls.isCalls || cls.isDispatches) ? 5 : 0,
        style: { stroke, strokeWidth: cls.isPayload ? 2 : (cls.isCalls || cls.isDispatches) ? 1.5 : 1 },
        labelStyle: { fill: labelFill, fontSize: 10 },
      };
    })
    .sort((a, b) => {
      const aPayload = a.style?.stroke === "#f472b6" ? 1 : 0;
      const bPayload = b.style?.stroke === "#f472b6" ? 1 : 0;
      return aPayload - bPayload;
    });

  return { nodes: componentNodes, edges };
}

export function buildFlowGraph(
  data: SysVistaOutput,
  activeKinds: Set<ComponentKind>,
): { nodes: Node[]; edges: Edge[] } {
  const flowEdges = data.edges.filter((e) => e.label && FLOW_LABELS.has(e.label));

  const flowNodeIds = flowEdges.reduce((acc, e) => {
    acc.add(e.from_id);
    acc.add(e.to_id);
    return acc;
  }, new Set<string>());

  const filteredComponents = data.components.filter(
    (c) => activeKinds.has(c.kind) && flowNodeIds.has(c.id),
  );
  const visibleIds = new Set(filteredComponents.map((c) => c.id));

  const uniqueEdges = deduplicateEdges(
    flowEdges.filter((e) => visibleIds.has(e.from_id) && visibleIds.has(e.to_id)),
  );

  // Only keep nodes that have at least one visible edge
  const connectedIds = uniqueEdges.reduce((acc, e) => {
    acc.add(e.from_id);
    acc.add(e.to_id);
    return acc;
  }, new Set<string>());
  const connectedComponents = filteredComponents.filter((c) => connectedIds.has(c.id));

  const positions = dagreLayout(connectedComponents, uniqueEdges, "LR", 50, 100);
  const visibleFlowEdges = flowEdges.filter((e) => visibleIds.has(e.from_id) && visibleIds.has(e.to_id));
  const hubMap = detectHubs(connectedComponents, visibleFlowEdges);
  const clusterMap = classifyComponents(connectedComponents);

  const nodes: Node<GraphNode>[] = connectedComponents.map((comp) =>
    toComponentNode(comp, positions.get(comp.id) ?? { x: 0, y: 0 }, hubMap, clusterMap, "LR"),
  );

  const edges: Edge[] = uniqueEdges
    .map((e, i) => {
      const cls = classifyEdge(e.labels);
      const { stroke, labelFill } = edgeStroke(cls);
      return {
        id: `fe-${i}`,
        source: e.from_id,
        target: e.to_id,
        label: formatEdgeLabel(e),
        animated: true,
        zIndex: cls.isPayload ? 10 : (cls.isCalls || cls.isDispatches) ? 5 : 0,
        style: { stroke, strokeWidth: cls.isPayload ? 2.5 : (cls.isCalls || cls.isDispatches) ? 2 : 1.5 },
        labelStyle: { fill: labelFill, fontSize: 10 },
      };
    })
    .sort((a, b) => {
      const aPayload = a.style?.stroke === "#f472b6" ? 1 : 0;
      const bPayload = b.style?.stroke === "#f472b6" ? 1 : 0;
      return aPayload - bPayload;
    });

  return { nodes, edges };
}
