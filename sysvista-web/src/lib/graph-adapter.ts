import dagre from "@dagrejs/dagre";
import type { Node, Edge } from "@xyflow/react";
import type { SysVistaOutput, DetectedComponent, DetectedEdge, ComponentKind } from "../types/schema";
import { KIND_NODE_SIZE } from "./design-tokens";
import type { Manifest, Snapshot } from "../types/v2";
import type { ProjectedScope } from "./projection/types";

interface HubInfo { tier: "high" | "medium" | "normal"; degree: number }

const detectHubs = (components: DetectedComponent[], edges: DetectedEdge[]): Map<string, HubInfo> => {
  const degrees = new Map(components.map(({ id }) => [id, 0]));
  for (const edge of edges) {
    degrees.set(edge.from_id, (degrees.get(edge.from_id) ?? 0) + 1);
    degrees.set(edge.to_id, (degrees.get(edge.to_id) ?? 0) + 1);
  }
  const values = [...degrees.values()];
  const mean = values.length ? values.reduce((sum, value) => sum + value, 0) / values.length : 0;
  const variance = values.length ? values.reduce((sum, value) => sum + (value - mean) ** 2, 0) / values.length : 0;
  const deviation = Math.sqrt(variance);
  return new Map([...degrees].map(([id, degree]) => [id, { degree, tier: degree > mean + 2 * deviation ? "high" : degree > mean + deviation ? "medium" : "normal" }]));
};

const COMPONENT_KINDS = new Set<ComponentKind>(["model", "service", "transport", "transform", "prompt"]);

export function projectedScopeToGraphInput(snapshot: Snapshot, projection: ProjectedScope): SysVistaOutput {
  const manifest = snapshot.manifest as Manifest;
  const visible = new Set(projection.children.map(({ id }) => id));
  const childComponents = (snapshot.entities ?? []).filter(({ id, declaration_kind }) =>
    visible.has(id) && COMPONENT_KINDS.has(declaration_kind as ComponentKind)).map((entity) => {
      const legacy = entity.attributes as Partial<DetectedComponent> | undefined;
      const file = snapshot.source_files?.find(({ id }) => id === entity.file_id);
      return {
        ...(legacy ?? {}), id: entity.id, name: String(entity.name),
        kind: entity.declaration_kind as ComponentKind,
        language: legacy?.language ?? String(file?.language ?? "unknown"),
        source: legacy?.source ?? { file: String(file?.path ?? entity.file_id), line_start: Number(entity.span.start_line), line_end: Number(entity.span.end_line) },
        metadata: legacy?.metadata ?? {},
      };
    });
  const boundaryByTarget = new Map(
    projection.boundaryNodes.map((boundary) => [boundary.externalTargetId, boundary.id]),
  );
  const boundaryComponents: DetectedComponent[] = projection.boundaryNodes.map((boundary) => ({
    id: boundary.id,
    name: boundary.name,
    kind: "service",
    language: "external",
    source: { file: String(boundary.externalTargetId) },
    metadata: { projection_boundary: "true", external_target_id: boundary.externalTargetId },
  }));
  const components = [...childComponents, ...boundaryComponents];
  return {
    version: String(manifest.schema_version), scanned_at: String(manifest.scanned_at),
    root_dir: String(manifest.root), project_name: String(manifest.repository),
    detected_languages: [...new Set(components.map(({ language }) => language))], components,
    edges: projection.relationships.map((relationship) => ({
      from_id: boundaryByTarget.get(relationship.source) ?? relationship.source,
      to_id: boundaryByTarget.get(relationship.target) ?? relationship.target,
      label: String(relationship.kind),
    })),
    workflows: [],
    scan_stats: { files_scanned: snapshot.source_files?.length ?? 0, files_skipped: 0, scan_duration_ms: 0 },
  };
}

export interface GraphNode extends Record<string, unknown> {
  component: DetectedComponent;
  hubTier: HubInfo["tier"];
  degree: number;
  highlighted?: boolean;
  direction?: "TB" | "LR";
}

const KIND_CONFIG = KIND_NODE_SIZE;

export const FLOW_LABELS = new Set(["handles", "persists", "transforms", "consumes", "produces", "calls", "dispatches"]);
const PAYLOAD_LABELS = new Set(["consumes", "produces"]);

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
  direction?: "TB" | "LR",
): Node<GraphNode> => {
  const hub = hubMap.get(comp.id) ?? { tier: "normal" as const, degree: 0 };
  return {
    id: comp.id,
    type: comp.metadata.projection_boundary === "true" ? "boundary" : comp.kind,
    position,
    data: {
      component: comp,
      hubTier: hub.tier,
      degree: hub.degree,
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
  const payload = e.payload_types.length > 0 ? `[${e.payload_types.join(", ")}]` : "";
  if (!base) return payload || undefined;
  return payload ? `${base} ${payload}` : base;
};

// --- Layout ---

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
  const filteredComponents = data.components.filter(
    (component) => component.metadata.projection_boundary === "true" || activeKinds.has(component.kind),
  );
  const visibleIds = new Set(filteredComponents.map((c) => c.id));
  const filteredEdges = data.edges.filter((e) => visibleIds.has(e.from_id) && visibleIds.has(e.to_id));
  const uniqueEdges = deduplicateEdges(filteredEdges);

  const hubMap = detectHubs(filteredComponents, filteredEdges);

  const positions = dagreLayout(filteredComponents, uniqueEdges, "TB", 60, 80);
  const componentNodes: Node[] = filteredComponents.map((comp) =>
    toComponentNode(comp, positions.get(comp.id) ?? { x: 0, y: 0 }, hubMap),
  );

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

  const nodes: Node<GraphNode>[] = connectedComponents.map((comp) =>
    toComponentNode(comp, positions.get(comp.id) ?? { x: 0, y: 0 }, hubMap, "LR"),
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
