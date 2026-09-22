interface LineGutterProps {
  count: number;
  highlighted?: { start: number; end: number };
}

export function LineGutter({ count, highlighted }: LineGutterProps) {
  return (
    <div aria-hidden="true" className="select-none border-r border-[var(--border)] bg-[var(--surface-raised)] py-3 text-right font-mono text-xs text-[var(--muted)]">
      {Array.from({ length: count }, (_, index) => index + 1).map((line) => (
        <div key={line} className={`px-2 leading-5 ${highlighted && line >= highlighted.start && line <= highlighted.end ? "bg-amber-200/60 text-amber-900" : ""}`}>{line}</div>
      ))}
    </div>
  );
}
