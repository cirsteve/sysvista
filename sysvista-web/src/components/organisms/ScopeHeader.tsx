interface ScopeHeaderProps { title: string; visible: number; total: number }

export function ScopeHeader({ title, visible, total }: ScopeHeaderProps) {
  return <div className="flex items-baseline gap-3"><h2 className="font-semibold">{title}</h2><span className="text-xs text-[var(--muted)]">{visible} visible / {total} total</span></div>;
}
