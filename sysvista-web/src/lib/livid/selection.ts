import type { DiagramSelection, DiagramFocus } from "@rankonelabs/livid-react";
import type { StateFrame, NodeId, EdgeId } from "@rankonelabs/livid-core";
import type { DiagramSpec } from "./types";

export function resolveSelection(spec: DiagramSpec, selectedId: string | null): DiagramSelection | null {
  if (!selectedId) return null;
  const node = spec.nodes.find(({ id }) => id === selectedId);
  if (node) return { kind: "node", id: node.id as NodeId, detail: node.details };
  const edge = spec.edges.find((candidate) => candidate.id === selectedId ||
    (candidate.presentation === "aggregate-edge" && candidate.details.relationshipIds.includes(selectedId)));
  return edge ? { kind: "edge", id: edge.id as EdgeId, detail: edge.details } : null;
}

export function resolveFocus(spec: DiagramSpec, id: string | null): DiagramFocus | null {
  const selection = resolveSelection(spec, id);
  return selection ? { kind: selection.kind, id: selection.id } as DiagramFocus : null;
}

export function selectionFrame(spec: DiagramSpec, selectedIds: readonly string[]): StateFrame {
  const nodes: Record<string, string> = {};
  const edges: Record<string, string> = {};
  for (const id of selectedIds) {
    const selection = resolveSelection(spec, id);
    if (selection?.kind === "node") nodes[selection.id] = "highlight";
    if (selection?.kind === "edge") edges[selection.id] = "highlight";
  }
  return { __brand: "StateFrame", nodes, edges } as StateFrame;
}
