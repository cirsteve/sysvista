import { useEffect, useRef } from "react";

interface ResizeHandleProps { onResize: (delta: number) => void; width: number }

export function ResizeHandle({ onResize, width }: ResizeHandleProps) {
  const start = useRef<number | null>(null);
  const handler = useRef(onResize);
  useEffect(() => { handler.current = onResize; }, [onResize]);
  useEffect(() => {
    const move = (event: PointerEvent) => {
      if (start.current === null) return;
      handler.current(start.current - event.clientX);
      start.current = event.clientX;
    };
    const up = () => { start.current = null; };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    return () => { window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", up); };
  }, []);
  return <div role="separator" tabIndex={0} aria-orientation="vertical" aria-label="Resize inspector" aria-valuemin={240} aria-valuemax={560} aria-valuenow={width}
    className="absolute inset-y-0 left-0 w-2 cursor-col-resize hover:bg-sky-500 focus:bg-sky-500 focus:outline-2 focus:outline-sky-400"
    onPointerDown={(event) => { start.current = event.clientX; }}
    onKeyDown={(event) => { if (event.key === "ArrowLeft" || event.key === "ArrowRight") { event.preventDefault(); onResize(event.key === "ArrowLeft" ? 16 : -16); } }} />;
}
