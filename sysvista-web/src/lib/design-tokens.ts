import type { ComponentKind, StepType } from "../types/schema";

export const KIND_COLORS: Record<ComponentKind, {
  bg: string;
  text: string;
  border: string;
  dot: string;
  hex: string;
}> = {
  model: {
    bg: "bg-blue-500/20",
    text: "text-blue-400",
    border: "border-blue-500/30",
    dot: "bg-blue-400",
    hex: "#3b82f6",
  },
  service: {
    bg: "bg-green-500/20",
    text: "text-green-400",
    border: "border-green-500/30",
    dot: "bg-green-400",
    hex: "#22c55e",
  },
  transport: {
    bg: "bg-orange-500/20",
    text: "text-orange-400",
    border: "border-orange-500/30",
    dot: "bg-orange-400",
    hex: "#f97316",
  },
  transform: {
    bg: "bg-purple-500/20",
    text: "text-purple-400",
    border: "border-purple-500/30",
    dot: "bg-purple-400",
    hex: "#a855f7",
  },
};

export const STEP_TYPE_COLORS: Record<StepType, { text: string; bg: string; label: string }> = {
  entry:    { text: "text-orange-400", bg: "bg-orange-500/20", label: "Entry" },
  call:     { text: "text-green-400",  bg: "bg-green-500/20",  label: "Call" },
  persist:  { text: "text-blue-400",   bg: "bg-blue-500/20",   label: "Persist" },
  dispatch: { text: "text-amber-400",  bg: "bg-amber-500/20",  label: "Dispatch" },
  response: { text: "text-cyan-400",   bg: "bg-cyan-500/20",   label: "Response" },
};

/** Node sizing config for graph layout (used by graph-adapter) */
export const KIND_NODE_SIZE: Record<ComponentKind, { width: number; height: number }> = {
  model:     { width: 180, height: 60 },
  service:   { width: 180, height: 60 },
  transport: { width: 200, height: 60 },
  transform: { width: 180, height: 60 },
};
