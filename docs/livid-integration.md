# Livid integration

SysVista owns the `ScopeRenderer` interface in `sysvista-web/src/lib/livid/types.ts`. Livid is a required build-time dependency: Vite statically bundles the renderer and its interaction CSS. Livid-specific registration, validation, normalization, and layout are isolated in `adapter.ts`; if Livid rejects a scope at runtime, the viewer retains the fixture-compatible surface and shows a visible diagnostic.

## Tested pairing

| Item | Recorded value |
| --- | --- |
| npm dependency `@rankonelabs/livid-core` | `0.3.0` (exact pin) |
| npm dependency `@rankonelabs/livid-react` | `0.2.0` (exact pin) |
| Reviewed source workspace | `/home/steve/codes/rol/livid` |
| Reviewed source commit | `96a16e8772cdd921267453011eae63f6dc941bb4` |
| Package versions recorded at that source pairing | `@rankonelabs/livid-core` `0.2.2`; `@rankonelabs/livid-react` `0.1.1` |
| Dependency semantics profile | `dependency` |
| Deferred-child key | `scope:<ScopeId>` |

CI installs the published exact pins with `npm ci`; it does not require a sibling checkout. The source pairing above records the workspace and commit supplied during integration review, while the npm rows record the subsequently published API-compatible packages used by this build. The root-with-three-scopes fixture is validated, normalized, and laid out through the real packages in `spec.test.ts`; validation failures become visible SysVista `Diagnostic` values and retain the fixture-compatible surface.

The peer versions used by SysVista are React `^19.2.0` and `@xyflow/react` `^12.10.0`.
