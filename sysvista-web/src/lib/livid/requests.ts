import type { Diagnostic, ScopeId } from "../../types/v2";

export const scopeRenderFailureDiagnostic = (cause: unknown): Diagnostic => ({
  kind: "warning",
  message: `Scope rendering failed; showing the fixture-compatible surface (${cause instanceof Error ? cause.message : String(cause)})`,
});

export interface ScopeRequestKey {
  snapshotId: string;
  scopeId: ScopeId;
  requestId: string;
}

const sameRequest = (left: ScopeRequestKey, right: ScopeRequestKey) =>
  left.snapshotId === right.snapshotId &&
  left.scopeId === right.scopeId &&
  left.requestId === right.requestId;

/** Coordinates async scope rendering and prevents superseded responses from committing. */
export class ScopeRequestCoordinator<T> {
  private current: ScopeRequestKey | null = null;

  begin(key: ScopeRequestKey) {
    this.current = key;
  }

  isCurrent(key: ScopeRequestKey) {
    return this.current !== null && sameRequest(this.current, key);
  }

  commit(key: ScopeRequestKey, value: T, replace: (value: T) => void): boolean {
    if (!this.isCurrent(key)) return false;
    replace(value);
    return true;
  }

  clear(key?: ScopeRequestKey) {
    if (!key || this.isCurrent(key)) this.current = null;
  }
}
