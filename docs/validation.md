# Validation and scale measurements

## Relationship corpus

`scripts/validate-corpus.sh` scans every labeled project in `corpus/cases`, runs the
shared snapshot validator, and reports precision and recall for each relationship kind
and origin in each case. The first baseline reported `1.000` for all kinds, but ten of
the twelve had no expectations and scored 0/0 as perfect. The corpus now includes
Python and Rust heuristic cases, reports a zero denominator as unmeasured, and lists
`contains` and `depends_on` as unmeasured because nothing emits them. Resolved imports
and calls measure `1.000`; the heuristic kinds measure what the detectors actually find.
Floors per kind and origin are in `corpus/floors.json`; the policy is documented in
`corpus/README.md`.

## Reference machine

- Measurement date: 2026-09-22
- Hostname: `otto`
- CPU: AMD Ryzen 7 PRO 6850U with Radeon Graphics
- RAM: 30.1 GiB
- OS: Linux
- Node: 22.21.1
- Fixture seed: `20260922`
- Analyzer heap: 8192 MiB (`SYSVISTA_ANALYZER_HEAP_MB=8192`)

## Scale results

The fixture generator creates deterministic, uncommitted input under a temporary
directory. Scan input is split into independent 5,000-relationship TypeScript projects;
the projection snapshot has three visible owners, matching the c4 warm-scope benchmark
topology. `scripts/measure-scale.sh` builds the release CLI, verifies that the emitted
graph contains the requested relationship count, measures the directory bundle size, and
reports the median of three warm projection runs.

| Relationships | CLI scan | Bundle size | Warm projection |
| ---: | ---: | ---: | ---: |
| 10,000 | 2,350 ms | 22,639,810 bytes | 8.1 ms |
| 100,000 | 63,073 ms | 226,753,400 bytes | 66.6 ms |
| 250,000 | 696,479 ms | 601,483,188 bytes | 223.7 ms |

The 250k scan exceeds the default Node 4 GiB heap: the reference run failed during
analysis with an out-of-memory diagnostic after roughly 32 seconds. The recorded 250k
number therefore uses the documented 8 GiB analyzer heap. This is a measured scale risk,
not an optimization or a claim that default configuration currently handles that input.

The c4 projection warning is now a hard assertion in
`sysvista-web/src/lib/projection/project.bench.ts`: the median warm 250k projection must
complete in at most 300 ms. The isolated c4 benchmark measured 264.4 ms and the generated
scale snapshot measured 223.7 ms; both pass. Generated fixtures are not committed.

Reproduce all measurements from the repository root:

```sh
scripts/measure-scale.sh
```

Use `SYSVISTA_SCALE_OUTPUT`, `SYSVISTA_SCALE_SEED`, `SYSVISTA_SCALE_SIZES`,
`SYSVISTA_ANALYZER_HEAP_MB`, or `SYSVISTA_VIEWER_HEAP_MB` to override the temporary
directory, seed, selected sizes, analyzer heap, or viewer heap while investigating
results.

## Hierarchy viewer measurements

PR B uses a parameterized source fixture scanned by the release CLI and then loaded by
`scripts/measure-viewer.ts`. The viewer report separates load and hierarchy indexing,
projection and navigation, validation and findings, and bounded diagram layout. The
small profile has 100 TypeScript files, about 1,000 declarations, and 2,500 relationships; the
large profile targets 10,000 TypeScript files, about 100,000 declarations, and 250,000
relationships. Timings are observations, not CI thresholds. The small profile runs on
push, while the large profile runs on dispatch and nightly.

| Profile | CLI scan | Bundle size | Load/index | Projection/navigation | Validation/findings | Bounded layout |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Small | 1,521 ms | 4,605,491 bytes | 18.0 ms | 9.3 ms | 44.1 ms | 96.2 ms |
| Large | 1,598,532 ms | 460,097,143 bytes | 2,863.7 ms | 1,089.9 ms | 1,697.6 ms | 154.3 ms |

The large report was produced with `SYSVISTA_SCALE_SIZES=large`, seed `20260922`,
`SYSVISTA_ANALYZER_HEAP_MB=12288`, and the default 8192 MiB viewer heap on the
reference machine. The generated source targeted 10,000 TypeScript files, 100,000
declarations, and 250,000 calls. The CLI bundle contained 10,201 inventoried files
(including package and config files), 130,000 entities, 260,000 relationships, and
201 findings. The 26.6-minute scan is a material runtime cost for manual and
nightly runs; earlier 8 GiB attempts were interrupted after about 25 minutes.

The small report was measured on the reference machine above with seed `20260922`.
Each package has its own `tsconfig.json`, so the real analyzer compiles independent
100-file projects rather than one monolithic TypeScript program.
Reproduce it with `SYSVISTA_SCALE_SIZES=small scripts/measure-scale.sh`; set
`SYSVISTA_SCALE_SIZES=large` for the full-size report. The generated source and JSON
reports remain outside the repository.

The integration loader also passed locally for `sysvista-web` and the repository root
as source and no-source folder and zip bundles. It runs the viewer's schema,
reference, hierarchy, and projection code. The integration acceptance criterion is
**zero CLI diagnostics except `analyzer_issue` and `payload_identity_conflict`**:
the repository scan includes intentionally malformed analyzer test fixtures, and
duplicate payload declarations in fixtures and generated types produce D7 ambiguity
notices. `scripts/load-bundle.ts` rejects every other CLI diagnostic, every viewer
validation error, duplicate ID, and absolute source or inventory path. The same
criterion applies to the `sysvista-web` scan and to all four folder/zip variants
with and without source.

## PR B GitHub Actions

GitHub Actions [CI run 55](https://github.com/cirsteve/sysvista/actions/runs/35820096575)
ran for PR #17 head `6615de273bfcb4cb2544f94887a908fc5fa76eeb` and completed
successfully. The PR base was `epic/drilldown-revref` for this run; that branch is
listed in the workflow's `pull_request.branches`, so the base did not prevent the
trigger. `pr_status` reported only CodeRabbit, but the Actions run and job records
confirm the following results:

| Job | Result |
| --- | --- |
| CLI to viewer bundle integration | Passed |
| Web (TypeScript) | Passed |
| CLI (Rust) | Passed |
| Generated v3 schema | Passed |

The scale job also passed its push profile (`small`). It did not run the `large`
profile, which is selected only for schedule or `workflow_dispatch` events.

## PR C integrated merge gate and reconciliation

`./scripts/merge-gate.sh` was run from the final PR C branch head after clean npm installs and a Cargo clean build. The final run exited zero. It covered analyzer and web typechecks, Rust and TypeScript tests (including jsdom), schema regeneration with no diff, real `sysvista-web` and repository-root scans, four viewer-loaded folder/zip variants per root, ID and reference checks, relative source and inventory paths, equal IDs across two scan roots with the same origin, the small scale run, and rejected truncated zip, changed source hash and oversized entry cases. The gate is committed and exposed as a `workflow_dispatch` CI job. This evidence is for the PR C head; the operator decides merge readiness and can rerun it on the later integration head.

The original parent brief's D1–D8 wording is not included in this cohort work order. The entries below reconcile the decisions explicitly identified in the supplied cohort brief; decisions whose original wording cannot be reconstructed are marked out of scope rather than assigned an invented interpretation.

| Brief decision | Cohort and evidence | Resolution |
| --- | --- | --- |
| D1 | Original decision text not supplied in this cohort brief. | Out of scope for a factual decision-level claim; PR B and C requirements are mapped below. |
| D2 | PR B CLI-side contract: `sysvista-cli/src/output/v2/`, `sysvista-cli/src/discovery/config.rs`; PR C manifest snapshot and schema. | Implemented on the CLI side, as the cohort brief specifies. |
| D3 | PR B single-owner hierarchy: `sysvista-cli/src/hierarchy/`, `tests/hierarchy.rs`, viewer projection tests. | One logical module owner per file. |
| D4 | Original decision text not supplied in this cohort brief. | Out of scope for a factual decision-level claim; PR B and C requirements are mapped below. |
| D5 | PR B pinned Livid packages and `sysvista-web/src/lib/livid/`; `docs/livid-integration.md`. | No Livid package update in PR C. |
| D6 | Original decision text not supplied in this cohort brief. | Out of scope for a factual decision-level claim; PR B and C requirements are mapped below. |
| D7 | PR B payload identity diagnostics in `tests/analyzer_merge.rs`; `scripts/load-bundle.ts` accepts only the documented fixture notices. | Same-named payload declarations remain an explicit ambiguity diagnostic. |
| D8 | PR B CLI-side hierarchy and bundle contract, PR C identity and manifest changes. | Implemented on the CLI side, as the cohort brief specifies. |

| Requirement | Cohort and evidence | Status |
| --- | --- | --- |
| Start from merged PR B and land ordered work items | PR C began at merge commit `e11cb0a`; commits for scan metadata, workflow cleanup and gate followed in order. | Met. |
| Two same-named locals carry `is_local` and are hidden | PR C analyzer fixture `locals.ts`, `sysvista-cli/tests/unique_ids.rs`, `sysvista-web/src/lib/projection/project.test.ts`. | Met. |
| XDG then HOME cache, stable origin/path IDs, one read and line counts | PR C `analyzer/spawn.rs`, `build.rs`, `output/v2/ids.rs`, `scanner/mod.rs`, `scanner/file_walker.rs`, `tests/determinism.rs`. | Met. |
| Manifest config snapshot, Claim alignment and generated schema | PR C `Manifest.config_snapshot`, `types/v2.ts`, regenerated schema and types; schema no-diff gate. | Met. |
| v2 Make scan, committed fixture and public sample removal | PR C `Makefile`, `App.tsx`, committed projection fixture, removed public JSON; folder and zip loaded. | Met. |
| Bounded archive source re-reads | PR B streaming archive cap; PR C `archive.ts` starts only the requested source stream, with a second-read test. | Met. |
| Remove three unused v1 files | `slices.ts` is imported by `project.test.ts`; `design-tokens.ts` and `types/schema.ts` have live imports in app, loader and v1 adapter code. | Infeasible under the required no-importer precondition; files retained. |
| README, USAGE and contract documentation | PR C `README.md`, `USAGE.md`, both `CLAUDE.md` files, identity, analyzer, bundle and Livid docs. | Met. |
| Integrated gate, clean installs, real bundles, corrupt/oversized rejection | PR C `scripts/merge-gate.sh`, CI dispatch job, final zero exit reported above. | Met on PR C head; integration-head rerun belongs to the operator. |
