import { useState } from "react";
import { useSource } from "../../hooks/useSource";
import type { LoadedSnapshot } from "../../lib/loader";
import type { DiagramEdge, DiagramNode } from "../../lib/livid/types";
import type { FileId } from "../../types/v2";
import { selectEvidenceComposition } from "../../lib/selectors";
import { EvidenceComposition } from "../molecules/EvidenceComposition";
import { InspectorSection } from "../molecules/InspectorSection";
import { ResizeHandle } from "../molecules/ResizeHandle";
import { SourceView } from "./SourceView";

interface InspectorProps { item: DiagramNode | DiagramEdge | null; loaded: LoadedSnapshot | null }

export function Inspector({ item, loaded }: InspectorProps) {
  const [width, setWidth] = useState(320);
  const fileId = item?.presentation === "symbol" ? item.details.fileId : item?.presentation === "file" ? item.id as FileId : undefined;
  const span = item?.presentation === "symbol" ? item.details.span : undefined;
  const file = loaded?.snapshot.source_files?.find(({ id }) => id === fileId);
  const source = useSource(loaded, fileId);
  return (
    <aside className="relative shrink-0 border-l border-[var(--border)] bg-[var(--surface)]" style={{ width }}>
      <ResizeHandle width={width} onResize={(delta) => setWidth((value) => Math.min(560, Math.max(240, value + delta)))} />
      <div className="border-b border-[var(--border)] px-4 py-3 font-semibold">Inspector</div>
      {!item ? <p className="p-4 text-sm text-[var(--muted)]">Select a module, file, symbol, boundary, or dependency.</p> : (
        <>
          <InspectorSection title={item.presentation === "aggregate-edge" ? "Aggregate edge" : item.presentation}>
            <p className="font-medium">{item.label}</p>
            <p className="break-all font-mono text-xs text-[var(--muted)]">{item.id}</p>
          </InspectorSection>
          {item.presentation === "aggregate-edge" && (
            <InspectorSection title="Evidence composition"><EvidenceComposition counts={selectEvidenceComposition(item)} /></InspectorSection>
          )}
          <InspectorSection title="Details">
            {Object.entries(item.details).map(([key, value]) => <div key={key}><span className="text-[var(--muted)]">{key}: </span><span className="break-all">{typeof value === "object" ? JSON.stringify(value) : String(value)}</span></div>)}
          </InspectorSection>
          {(item.presentation === "symbol" || item.presentation === "file") && <SourceView source={source} language={file?.language} span={span} />}
        </>
      )}
    </aside>
  );
}
