import { useEffect, useRef } from "react";
import type { DiagramSpec } from "../lib/livid/types";

interface KeyboardNavigation {
  spec: DiagramSpec | null;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onDescend: (deferredChildKey: string) => void;
  onBack: () => void;
}

export function useKeyboardNavigation(options: KeyboardNavigation) {
  const latest = useRef(options);
  useEffect(() => { latest.current = options; }, [options]);
  useEffect(() => {
    const keydown = (event: KeyboardEvent) => {
      const { spec, selectedId, onSelect, onDescend, onBack } = latest.current;
      if (!spec || event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) return;
      const nodes = spec.overBudget ? (spec.items ?? []).map((item) => ({
        id: item.id, deferredChildKey: item.scopeId ? `scope:${item.scopeId}` : undefined,
      })) : spec.nodes;
      if (event.key === "Backspace") { onBack(); event.preventDefault(); return; }
      if (!nodes.length) return;
      const current = Math.max(0, nodes.findIndex(({ id }) => id === selectedId));
      if (["ArrowRight", "ArrowDown", "ArrowLeft", "ArrowUp"].includes(event.key)) {
        const delta = event.key === "ArrowRight" || event.key === "ArrowDown" ? 1 : -1;
        onSelect(nodes[(current + delta + nodes.length) % nodes.length]?.id ?? null);
        event.preventDefault();
      } else if (event.key === "Enter") {
        const node = nodes[current];
        if (node?.deferredChildKey) onDescend(node.deferredChildKey);
        event.preventDefault();
      }
    };
    window.addEventListener("keydown", keydown);
    return () => window.removeEventListener("keydown", keydown);
  }, []);
}
