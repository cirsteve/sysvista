const items = [
  ["Module", "bg-indigo-500"], ["File", "bg-emerald-500"], ["Symbol", "bg-sky-500"],
  ["Boundary", "bg-amber-500"], ["Aggregate dependency", "bg-rose-500"],
] as const;

export function Legend() {
  return <div className="flex flex-wrap items-center gap-3 border-t border-[var(--border)] bg-[var(--surface)] px-4 py-2">{items.map(([label, color]) => <span key={label} className="flex items-center gap-1.5 text-xs text-[var(--muted)]"><i className={`h-2.5 w-2.5 rounded-full ${color}`} />{label}</span>)}</div>;
}
