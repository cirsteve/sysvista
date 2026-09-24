import { useEffect, useRef } from "react";

interface ResizeHandleProps { onResize: (delta: number) => void }

export function ResizeHandle({ onResize }: ResizeHandleProps) {
  const start = useRef<number | null>(null);
  useEffect(() => {
    const move = (event: PointerEvent) => {
      if (start.current === null) return;
      onResize(start.current - event.clientX);
      start.current = event.clientX;
    };
    const up = () => { start.current = null; };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    return () => { window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", up); };
  }, [onResize]);
  return <div role="separator" aria-orientation="vertical" aria-label="Resize inspector" className="absolute inset-y-0 left-0 w-1 cursor-col-resize hover:bg-sky-500" onPointerDown={(event) => { start.current = event.clientX; }} />;
}
