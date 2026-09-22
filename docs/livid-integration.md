# Livid integration

SysVista owns the `ScopeRenderer` interface in `sysvista-web/src/lib/livid/types.ts`. Livid-specific loading, registration, and validation are isolated in `adapter.ts`; the viewer falls back to the fixture-backed renderer with a visible diagnostic if that boundary is unavailable or rejects a scope.

## Tested pairing

| Item | Recorded value |
| --- | --- |
| `@rankonelabs/livid-core` | `file:../../livid/packages/core` (workspace package version unavailable) |
| `@rankonelabs/livid-react` | `file:../../livid/packages/react` (workspace package version unavailable) |
| Livid workspace commit | unavailable — the sibling `../../livid` workspace was absent in this checkout |
| Dependency semantics profile | `sysvista-dependency-v1` |
| Deferred-child key | `scope:<ScopeId>` |

The real-package validation/interaction round trip is therefore **BLOCKED on the Livid companion workspace**. This does not block the fake renderer, projection golden, review UI, or downstream lens work. Once the workspace is supplied, replace the two unavailable values above with its package versions and commit after the root fixture passes Livid validation (including static-call fan-out and a cycle) and deferred descent invokes `onDescend`.

The peer versions used by SysVista are React `^19.2.0` and `@xyflow/react` `^12.10.0`.
