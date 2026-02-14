import dagre from "@dagrejs/dagre";
import type { Node, Edge } from "@xyflow/react";
import type { SysVistaOutput, DetectedComponent, ComponentKind } from "../types/schema";
import { classifyComponents, detectHubs, type HubInfo } from "./clustering";
import type { ClusterLabelData } from "../components/nodes/ClusterLabelNode";

export interface GraphNode extends Record<string, unknown> {
  component: DetectedComponent;
  hubTier: HubInfo["tier"];
  degree: number;
  cluster: string;
  highlighted?: boolean;
}

export interface GroupLabelNode extends Record<string, unknown> {
  label: string;
  count: number;
  kind: ComponentKind;
}

const KIND_CONFIG: Record<ComponentKind, { color: string; width: number; height: number }> = {
  model:     { color: "#3b82f6", width: 180, height: 60 },
  service:   { color: "#22c55e", width: 180, height: 60 },
  transport: { color: "#f97316", width: 200, height: 60 },
  transform: { color: "#a855f7", width: 180, height: 60 },
};

// Dagre can't handle dense graphs — fall back to cluster grid layout above this threshold
const MAX_DAGRE_EDGES = 2000;

function clusterGridLayout(
  components: DetectedComponent[],
  clusterMap: Map<string, string>,
  hubMap: Map<string, HubInfo>,
): { positions: Map<string, { x: number; y: number }>; headerNodes: Node<ClusterLabelData>[] } {
  // Group by cluster
  const clusters = new Map<string, DetectedComponent[]>();
  for (const c of components) {
    const cluster = clusterMap.get(c.id) ?? "Other";
    let list = clusters.get(cluster);
    if (!list) {
      list = [];
      clusters.set(cluster, list);
    }
    list.push(c);
  }

  // Sort clusters: largest first, "Other" last
  const sortedClusters = [...clusters.entries()].sort((a, b) => {
    if (a[0] === "Other") return 1;
    if (b[0] === "Other") return -1;
    return b[1].length - a[1].length;
  });

  // Sort components within each cluster: hubs first (by degree desc)
  for (const [, comps] of sortedClusters) {
    comps.sort((a, b) => {
      const da = hubMap.get(a.id)?.degree ?? 0;
      const db = hubMap.get(b.id)?.degree ?? 0;
      return db - da;
    });
  }

  const positions = new Map<string, { x: number; y: number }>();
  const headerNodes: Node<ClusterLabelData>[] = [];
  const cols = Math.max(4, Math.ceil(Math.sqrt(components.length / 2)));
  const cellW = 240;
  const cellH = 100;
  const headerH = 50;
  const groupGap = 60;
  let offsetY = 0;

  for (const [clusterName, comps] of sortedClusters) {
    if (comps.length === 0) continue;

    headerNodes.push({
      id: `cluster-${clusterName}`,
      type: "clusterLabel",
      position: { x: 0, y: offsetY },
      data: { label: clusterName, count: comps.length },
      selectable: false,
      draggable: false,
    });

    offsetY += headerH;

    const rows = Math.ceil(comps.length / cols);
    for (let i = 0; i < comps.length; i++) {
      const col = i % cols;
      const row = Math.floor(i / cols);
      positions.set(comps[i].id, {
        x: col * cellW,
        y: offsetY + row * cellH,
      });
    }
    offsetY += rows * cellH + groupGap;
  }

  return { positions, headerNodes };
}

export function buildGraph(
  data: SysVistaOutput,
  activeKinds: Set<ComponentKind>,
): { nodes: Node[]; edges: Edge[] } {
  const filteredComponents = data.components.filter((c) =>
    activeKinds.has(c.kind),
  );
  const visibleIds = new Set(filteredComponents.map((c) => c.id));

  const filteredEdges = data.edges.filter(
    (e) => visibleIds.has(e.from_id) && visibleIds.has(e.to_id),
  );

  // Deduplicate edges between the same node pair
  const edgeMap = new Map<string, { from_id: string; to_id: string; labels: string[] }>();
  for (const e of filteredEdges) {
    const key = `${e.from_id}->${e.to_id}`;
    const existing = edgeMap.get(key);
    if (existing) {
      if (e.label && !existing.labels.includes(e.label)) {
        existing.labels.push(e.label);
      }
    } else {
      edgeMap.set(key, { from_id: e.from_id, to_id: e.to_id, labels: e.label ? [e.label] : [] });
    }
  }
  const uniqueEdges = [...edgeMap.values()];

  // Compute clustering and hub info
  const clusterMap = classifyComponents(filteredComponents);
  const hubMap = detectHubs(filteredComponents, filteredEdges);

  let componentNodes: Node<GraphNode>[];

  if (uniqueEdges.length > MAX_DAGRE_EDGES) {
    const { positions, headerNodes } = clusterGridLayout(filteredComponents, clusterMap, hubMap);

    componentNodes = filteredComponents.map((comp) => {
      const pos = positions.get(comp.id) ?? { x: 0, y: 0 };
      const hub = hubMap.get(comp.id) ?? { tier: "normal" as const, degree: 0 };
      return {
        id: comp.id,
        type: comp.kind,
        position: pos,
        data: {
          component: comp,
          hubTier: hub.tier,
          degree: hub.degree,
          cluster: clusterMap.get(comp.id) ?? "Other",
        },
      };
    });

    componentNodes = [...(headerNodes as Node[]), ...componentNodes];
  } else {
    const g = new dagre.graphlib.Graph();
    g.setDefaultEdgeLabel(() => ({}));
    g.setGraph({ rankdir: "TB", nodesep: 60, ranksep: 80 });

    for (const comp of filteredComponents) {
      const config = KIND_CONFIG[comp.kind];
      g.setNode(comp.id, { width: config.width, height: config.height });
    }

    for (const edge of uniqueEdges) {
      g.setEdge(edge.from_id, edge.to_id);
    }

    dagre.layout(g);

    componentNodes = filteredComponents.map((comp) => {
      const n = g.node(comp.id);
      const config = KIND_CONFIG[comp.kind];
      const hub = hubMap.get(comp.id) ?? { tier: "normal" as const, degree: 0 };
      return {
        id: comp.id,
        type: comp.kind,
        position: {
          x: n.x - config.width / 2,
          y: n.y - config.height / 2,
        },
        data: {
          component: comp,
          hubTier: hub.tier,
          degree: hub.degree,
          cluster: clusterMap.get(comp.id) ?? "Other",
        },
      };
    });
  }

  const FLOW_LABELS = new Set(["handles", "persists", "transforms"]);

  const edges: Edge[] = uniqueEdges.map((e, i) => {
    const label = e.labels.join(", ");
    const isFlow = e.labels.some((l) => FLOW_LABELS.has(l));
    return {
      id: `e-${i}`,
      source: e.from_id,
      target: e.to_id,
      label: label || undefined,
      animated: isFlow || e.labels.includes("calls"),
      style: { stroke: isFlow ? "#06b6d4" : "#6b7280" },
      labelStyle: { fill: isFlow ? "#67e8f9" : "#9ca3af", fontSize: 10 },
    };
  });

  return { nodes: componentNodes, edges };
}
