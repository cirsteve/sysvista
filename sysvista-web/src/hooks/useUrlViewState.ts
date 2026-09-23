import { useEffect, useRef, useState } from "react";
import { decodeViewState, encodeViewState } from "../lib/view-state/hash";
import type { ViewState } from "../lib/view-state/types";
import { useViewStore } from "../store/viewStore";

export function useUrlViewState() {
  const [hydrated, setHydrated] = useState(false);
  const view = useViewStore((state) => state.view);
  const replaceView = useViewStore((state) => state.replaceView);
  const addDiagnostic = useViewStore((state) => state.addDiagnostic);
  const previous = useRef<ViewState | null>(null);
  const restoring = useRef(false);

  useEffect(() => {
    const restore = () => {
      restoring.current = true;
      const decoded = decodeViewState(window.location.hash);
      previous.current = decoded.state;
      replaceView(decoded.state, false);
      decoded.diagnostics.forEach(addDiagnostic);
      setHydrated(true);
    };
    restore();
    window.addEventListener("popstate", restore);
    return () => window.removeEventListener("popstate", restore);
  }, [replaceView, addDiagnostic]);

  useEffect(() => {
    if (!hydrated) return;
    if (restoring.current) { restoring.current = false; previous.current = view; return; }
    const encoded = encodeViewState(view);
    if (window.location.hash === encoded) { previous.current = view; return; }
    const push = previous.current && (previous.current.scopeId !== view.scopeId || previous.current.lens !== view.lens);
    previous.current = view;
    if (push) window.history.pushState(null, "", encoded);
    else {
      const timer = window.setTimeout(() => {
        if (window.location.hash !== encoded) window.history.replaceState(null, "", encoded);
      }, 100);
      return () => window.clearTimeout(timer);
    }
  }, [hydrated, view]);
  return view;
}
