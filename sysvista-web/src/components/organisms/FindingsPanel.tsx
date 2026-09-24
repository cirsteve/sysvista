import type { EntityId, Finding, ScopeId, Snapshot } from "../../types/v2";

interface FindingNavigation {
  scopeId: ScopeId;
  entityIds: EntityId[];
}

interface FindingsPanelProps {
  snapshot: Snapshot | null;
  onNavigate: (target: FindingNavigation) => void;
}

function targetFor(snapshot: Snapshot, finding: Finding): FindingNavigation | null {
  const entities = new Map((snapshot.entities ?? []).map((entity) => [entity.id, entity]));
  if (finding.kind === "rule") {
    const rule = finding as unknown as { navigation_target: { scope_id: string; entity_id?: string | null }; affected_entity_ids?: string[] };
    const affected = (rule.affected_entity_ids ?? []) as EntityId[];
    const primary = rule.navigation_target.entity_id as EntityId | null | undefined;
    return { scopeId: rule.navigation_target.scope_id as ScopeId, entityIds: [...new Set([...(primary ? [primary] : []), ...affected])] };
  }
  if (finding.kind === "entity") {
    const entity = entities.get(finding.entity_id as EntityId);
    return entity ? { scopeId: entity.scope_id, entityIds: [entity.id] } : null;
  }
  if (finding.kind === "relationship") {
    const relationship = (snapshot.relationships ?? []).find(({ id }) => id === finding.relationship_id);
    const source = relationship ? entities.get(relationship.source) : undefined;
    return relationship && source ? { scopeId: source.scope_id, entityIds: [relationship.source, relationship.target] } : null;
  }
  return null;
}

export function FindingsPanel({ snapshot, onNavigate }: FindingsPanelProps) {
  const findings = (snapshot?.findings ?? []) as Finding[];
  return (
    <aside className="w-72 shrink-0 overflow-auto border-l border-[var(--border)] bg-[var(--surface)]">
      <div className="border-b border-[var(--border)] px-4 py-3 font-semibold">Findings <span className="text-xs text-[var(--muted)]">({findings.length})</span></div>
      {findings.length === 0 ? <p className="p-4 text-sm text-[var(--muted)]">No findings.</p> : findings.map((finding, index) => {
        const target = snapshot ? targetFor(snapshot, finding) : null;
        return <button key={"id" in finding ? String(finding.id) : `${finding.kind}-${index}`} type="button" disabled={!target} onClick={() => target && onNavigate(target)} className="block w-full border-b border-[var(--border)] p-3 text-left text-sm enabled:hover:bg-[var(--surface-raised)] disabled:opacity-60">
          <span className="block text-xs uppercase text-[var(--muted)]">{String(finding.kind)}</span>
          <span>{String(finding.message)}</span>
        </button>;
      })}
    </aside>
  );
}
