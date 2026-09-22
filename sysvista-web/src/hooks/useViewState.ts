import { useEffect, useState } from "react";
import { decodeViewState, encodeViewState } from "../lib/view-state/hash";
import { useViewStore } from "../store/viewStore";

export function useViewState() {
  const [hydrated, setHydrated] = useState(false);
  const view = useViewStore((state) => state.view);
  const replaceView = useViewStore((state) => state.replaceView);
  const addDiagnostic = useViewStore((state) => state.addDiagnostic);

  useEffect(() => {
    const restore = () => {
      const decoded = decodeViewState(window.location.hash, view);
      replaceView(decoded.state, false);
      decoded.diagnostics.forEach(addDiagnostic);
      setHydrated(true);
    };
    restore();
    window.addEventListener("hashchange", restore);
    return () => window.removeEventListener("hashchange", restore);
    // Initial hash is intentionally decoded once; subsequent changes arrive via hashchange.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [replaceView, addDiagnostic]);

  useEffect(() => {
    if (!hydrated) return;
    const encoded = encodeViewState(view);
    if (window.location.hash !== encoded) window.history.replaceState(null, "", encoded);
  }, [hydrated, view]);

  return view;
}
