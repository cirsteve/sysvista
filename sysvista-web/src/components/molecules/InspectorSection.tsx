import type { ReactNode } from "react";

interface InspectorSectionProps { title: string; children: ReactNode }

export function InspectorSection({ title, children }: InspectorSectionProps) {
  return (
    <section className="border-b border-[var(--border)] px-4 py-3">
      <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-[var(--muted)]">{title}</h3>
      <div className="space-y-2 text-sm">{children}</div>
    </section>
  );
}
