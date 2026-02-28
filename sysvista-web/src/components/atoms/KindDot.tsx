import type { ComponentKind } from "../../types/schema";
import { KIND_COLORS } from "../../lib/design-tokens";

interface KindDotProps {
  kind: ComponentKind;
  size?: "sm" | "md";
  shape?: "circle" | "square";
}

export function KindDot({ kind, size = "sm", shape = "circle" }: KindDotProps) {
  const sizeClass = size === "sm" ? "w-2 h-2" : "w-3 h-3";
  const shapeClass = shape === "circle" ? "rounded-full" : "rounded-sm";
  return (
    <span className={`${sizeClass} ${shapeClass} shrink-0 ${KIND_COLORS[kind].dot}`} />
  );
}
