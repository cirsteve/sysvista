import { useEffect } from "react";
import type { DiagramSpec } from "../lib/livid/types";

interface KeyboardNavigation {
  spec: DiagramSpec | null;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onDescend: (deferredChildKey: string) => void;
  onBack: () => void;
}

export function useKeyboardNavigation({ spec, selectedId, onSelect, onDescend, onBack }: KeyboardNavigation) {
  useEffect(() => {
    const keydown = (event: KeyboardEvent) => {
      if (!spec || event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) return;
      const current = Math.max(0, spec.nodes.findIndex(({ id }) => id === selectedId));
      if (["ArrowRight", "ArrowDown", "ArrowLeft", "ArrowUp"].includes(event.key)) {
        const delta = event.key === "ArrowRight" || event.key === "ArrowDown" ? 1 : -1;
        const next = (current + delta + spec.nodes.length) % spec.nodes.length;
        onSelect(spec.nodes[next]?.id ?? null);
        event.preventDefault();
      } else if (event.key === "Enter") {
        const node = spec.nodes[current];
        if (node?.deferredChildKey) onDescend(node.deferredChildKey);
        event.preventDefault();
      } else if (event.key === "Backspace") {
        onBack();
        event.preventDefault();
      }
    };
    window.addEventListener("keydown", keydown);
    return () => window.removeEventListener("keydown", keydown);
  }, [spec, selectedId, onSelect, onDescend, onBack]);
}
