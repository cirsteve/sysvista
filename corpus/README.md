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
scoring these labels. Precision and recall floors will be documented here after the first
complete corpus run measures them; later changes may only ratchet those floors upward.
