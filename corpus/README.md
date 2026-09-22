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
the same locator. `origin` is optional and constrains provenance when present. Array order
is not significant.

The relationship kinds are `imports`, `references`, `calls`, `contains`, `depends_on`,
`handles`, `persists`, `transforms`, `consumes`, `produces`, `dispatches`, and
`invokes_prompt`. The corpus validator runs the shared v2 snapshot validator before
scoring these labels. When both the expected and actual sets are empty, precision and
recall are defined as `1.0`. Run `scripts/validate-corpus.sh` from any directory to print
per-case and aggregate metrics and enforce the floors.

## Current floors

The first complete run measured `1.000` aggregate precision and recall for every kind.
The initial floor is that measurement minus a `0.020` margin. Floors are ratchets: later
changes may increase them but must not lower them.

| Relationship kind | Precision floor | Recall floor |
| --- | ---: | ---: |
| imports | 0.980 | 0.980 |
| references | 0.980 | 0.980 |
| calls | 0.980 | 0.980 |
| contains | 0.980 | 0.980 |
| depends_on | 0.980 | 0.980 |
| handles | 0.980 | 0.980 |
| persists | 0.980 | 0.980 |
| transforms | 0.980 | 0.980 |
| consumes | 0.980 | 0.980 |
| produces | 0.980 | 0.980 |
| dispatches | 0.980 | 0.980 |
| invokes_prompt | 0.980 | 0.980 |
