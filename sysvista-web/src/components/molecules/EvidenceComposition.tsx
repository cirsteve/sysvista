interface EvidenceCompositionProps { counts: Record<string, number> }

export function EvidenceComposition({ counts }: EvidenceCompositionProps) {
  return (
    <dl className="space-y-1">
      {Object.entries(counts).sort(([a], [b]) => a.localeCompare(b)).map(([origin, count]) => (
        <div key={origin} className="flex justify-between rounded bg-[var(--surface-raised)] px-2 py-1">
          <dt>{origin}</dt><dd className="font-mono">{count}</dd>
        </div>
      ))}
    </dl>
  );
}
