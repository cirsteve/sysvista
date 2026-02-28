import type { ComponentKind, StepType } from "../../types/schema";
import { KIND_COLORS, STEP_TYPE_COLORS } from "../../lib/design-tokens";

type BadgeVariant = "kind" | "count" | "type" | "step" | "field";

interface BadgeProps {
  label: string;
  variant: BadgeVariant;
  kind?: ComponentKind;
  stepType?: StepType;
  /** Custom color classes for "type" variant (e.g. consumes/produces badges) */
  colorClass?: string;
}

export function Badge({ label, variant, kind, stepType, colorClass }: BadgeProps) {
  const base = "inline-flex items-center text-xs font-medium rounded px-1.5 py-0.5";

  switch (variant) {
    case "kind": {
      const c = kind ? KIND_COLORS[kind] : undefined;
      return (
        <span className={`${base} border ${c ? `${c.text} ${c.bg} ${c.border}` : "text-gray-400 bg-gray-500/10 border-gray-500/30"}`}>
          {label}
        </span>
      );
    }
    case "count":
      return (
        <span className={`${base} rounded-full min-w-[1.25rem] text-center ${colorClass ?? "bg-gray-700 text-gray-300"}`}>
          {label}
        </span>
      );
    case "type":
      return (
        <span className={`${base} font-mono border ${colorClass ?? "bg-gray-800 text-gray-300 border-gray-700"}`}>
          {label}
        </span>
      );
    case "step": {
      const s = stepType ? STEP_TYPE_COLORS[stepType] : undefined;
      return (
        <span className={`${base} font-bold ${s ? `${s.text} ${s.bg}` : "text-gray-400 bg-gray-500/20"}`}>
          {label}
        </span>
      );
    }
    case "field":
      return (
        <span className={`${base} font-mono bg-gray-800 text-gray-300`}>
          {label}
        </span>
      );
  }
}
