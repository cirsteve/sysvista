import type { Diagnostic, ScopeId } from "../../types/v2";

export interface ScopeFallback {
  scopeId: ScopeId;
  diagnostic?: Diagnostic;
}

export function nearestValidScope(
  requested: ScopeId,
  valid: ReadonlySet<ScopeId>,
  parentByScope: ReadonlyMap<ScopeId, ScopeId>,
  root: ScopeId,
): ScopeFallback {
  let candidate: ScopeId | undefined = requested;
  const visited = new Set<ScopeId>();
  while (candidate && !visited.has(candidate)) {
    if (valid.has(candidate)) {
      return candidate === requested ? { scopeId: candidate } : {
        scopeId: candidate,
        diagnostic: { kind: "warning", message: `Scope ${requested} is unavailable; restored nearest ancestor ${candidate}` },
      };
    }
    visited.add(candidate);
    candidate = parentByScope.get(candidate);
  }
  return {
    scopeId: root,
    diagnostic: { kind: "warning", message: `Scope ${requested} is unavailable; restored root scope ${root}` },
  };
}
