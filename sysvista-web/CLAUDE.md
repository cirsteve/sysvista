# sysvista-web

React 19 / TypeScript viewer for SysVista schema version 3 folder and zip bundles. Install with `npm ci`, run `npm run dev`, build with `npm run build`, and run both unit and jsdom tiers with `npm test`. `npm run lint` checks the app. The dev app loads a committed v2 test fixture on startup; the import controls accept a folder bundle or zip.

`src/lib/loader.ts` validates schema and references before building a hierarchy index. `src/lib/bundle/` reads bounded archives and verifies source hashes. `src/lib/hierarchy/` and `src/lib/projection/` select physical and logical scope children and aggregate crossing relationships. Function-scoped `is_local` entities are hidden in the default projection. `src/hooks/useGraphData.ts` owns loaded data, selection, navigation, lens state and flow expansion; Zustand stores persistent view state. `src/components/organisms/` renders navigation, diagram, findings, source and inspector controls.

Livid is pinned through `@rankonelabs/livid-core` and `@rankonelabs/livid-react`. `src/lib/livid/adapter.ts` is the boundary to the renderer; rejected specs display a diagnostic and use the fixture-compatible fallback. Selection is expressed as stable entity or relationship IDs. The structural lens navigates one scope at a time; the flow lens expands from a selected entity with a bounded hop count. Source reads are lazy and verified against `source-index.json`.

`src/types/v2.generated.ts` comes from the CLI schema. `src/types/v2.ts` adds branded IDs and uses the generated Claim shape. Run `../scripts/regen-schema.sh` after CLI contract changes. Legacy v1 adapters remain in `src/lib/adapters/v1.ts` for old JSON imports; their types and tokens still have live importers.
