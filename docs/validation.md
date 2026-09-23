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
