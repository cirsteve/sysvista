import type { EntityId } from "../../types/v2";

export type FlowRoot =
  | { kind: "entity"; entityId: EntityId }
  | { kind: "payload"; contractName: string };

export interface UnknownContinuation {
  id: string;
  name: string;
  reason: "partial" | "unresolved" | "heuristic";
}

export interface FlowNode {
  id: EntityId;
  label: string;
  depth: number;
  branch: boolean;
  truncationCount: number;
  unknownContinuations: UnknownContinuation[];
}

export interface FlowEdge {
  id: string;
  source: EntityId;
  target: EntityId;
  backEdge: boolean;
  argumentPayloads: string[];
  returnPayloads: string[];
}

export interface FlowGraph {
  horizon: number;
  rootIds: EntityId[];
  nodes: FlowNode[];
  edges: FlowEdge[];
}
