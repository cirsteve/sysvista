import type { FlowGraph } from "./types";

export function boundFlowGraph(graph: FlowGraph, maxBreadth = 40, maxNodes = 300): FlowGraph {
  if (!Number.isFinite(graph.horizon)) throw new RangeError("flow hops must be finite");
  const ordered = [...graph.nodes].sort((a, b) => a.depth - b.depth || a.id.localeCompare(b.id));
  const countByDepth = new Map<number, number>();
  const kept = ordered.filter((node) => {
    const count = countByDepth.get(node.depth) ?? 0;
    if (count >= maxBreadth) return false;
    countByDepth.set(node.depth, count + 1);
    return true;
  }).slice(0, maxNodes);
  const ids = new Set(kept.map(({ id }) => id));
  const dropped = new Map<string, number>();
  for (const edge of graph.edges) {
    if (ids.has(edge.source) && !ids.has(edge.target)) dropped.set(edge.source, (dropped.get(edge.source) ?? 0) + 1);
  }
  return { ...graph, nodes: kept.map((node) => ({ ...node, truncationCount: node.truncationCount + (dropped.get(node.id) ?? 0) })),
    edges: graph.edges.filter((edge) => ids.has(edge.source) && ids.has(edge.target)),
    rootIds: graph.rootIds.filter((id) => ids.has(id)) };
}
