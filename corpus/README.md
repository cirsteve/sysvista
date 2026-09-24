# Validation corpus

The validation corpus is a set of small, labeled projects used to measure relationship
precision and recall. Each directory under `cases/` is an independent scan root and
contains:

- the source files needed to exercise one analysis behavior;
- an `expected.json` file containing the relationship labels for that project; and
- optional project configuration such as `tsconfig.json`, `package.json`, or
  `sysvista.toml` when the case depends on it.

Cases must be deterministic and self-contained. Do not install dependencies in a case or
commit generated scan output.

## `expected.json`

Every expectation file has a `relationships` object keyed by the v2 relationship kind.
Each kind is present even when its expected list is empty, because an empty list is an
explicit assertion that the scan must not produce that relationship kind.

Relationship endpoints use stable entity locators rather than generated IDs:

```json
{
  "relationships": {
    "calls": [
      {
        "source": {
          "file": "main.ts",
          "qualified_name": "run",
          "declaration_kind": "function",
          "discriminator": 0
        },
        "target": {
          "file": "worker.ts",
          "qualified_name": "work",
          "declaration_kind": "function",
          "discriminator": 0
        },
        "origin": "resolved"
      }
    ]
  }
}
```

`file`, `qualified_name`, and `declaration_kind` identify an endpoint within a case.
`discriminator` is optional and defaults to `0`; set it when declarations otherwise share
the same locator (it counts same-locator declarations in source order). `origin` is
required: `resolved` for the TypeScript analyzer, `heuristic` for the pattern detectors.
An expectation without an origin could be satisfied by a weaker one, so it is rejected.
Array order is not significant.

The relationship kinds are `imports`, `references`, `calls`, `contains`, `depends_on`,
`handles`, `persists`, `transforms`, `consumes`, `produces`, `dispatches`, and
`invokes_prompt`. The corpus validator runs the shared v2 snapshot validator before
scoring these labels. Run `scripts/validate-corpus.sh` from any directory to print
per-case and aggregate metrics and enforce the floors.

## Labelling heuristic kinds

Heuristic cases are labelled against what each kind means, not against what the
detectors happen to produce, so their measurements show the detectors' real reach:

| Kind | Meaning |
| --- | --- |
| `imports` | A module imports a named component, or a whole module (target `<module>`) |
| `references` | A component's source names a model type declared in another file |
| `calls` | A component calls another component |
| `handles` | A service and a transport are declared in the same file |
| `persists` | A transport, service or prompt body uses a model type |
| `transforms` | A transform body uses a model type |
| `consumes` | A model is a transport's request payload (model → transport) |
| `produces` | A model is a transport's response payload (transport → model) |
| `dispatches` | A transport or service schedules a component as background work |
| `invokes_prompt` | A service uses a prompt declared in its body |

## Floors

`floors.json` lists every kind as either measured or unmeasured.

- **Measured** kinds have a precision and recall floor per origin. Every measured
  kind/origin pair must be expected by at least one case; a floor without support
  fails. The same floors apply to each case and to the corpus aggregate.
- **Unmeasured** kinds carry the reason no case measures them. A case that labels or
  produces an unmeasured kind fails; measure it and move it to `measured` instead.
- A metric with a zero denominator is reported as `unmeasured`, never as `1.000`.
  Nothing produced means precision is unknown; nothing expected means recall is.
- A case may lower a floor for its own known weaknesses with a top-level `floors`
  object in `expected.json`, keyed by kind then origin. The Rust and Python heuristic
  cases do this for detector gaps the aggregate would otherwise hide.

Floors were set from a measured run minus a `0.020` margin. They are ratchets: later
changes may raise them but must not lower them. Resolved `imports` and `calls` hold at
`0.980`; the heuristic floors are the detectors' measured reach, which is low for
`references`, Python and Rust `imports`, and `consumes`.
