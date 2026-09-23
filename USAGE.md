# SysVista v2 usage

From the repository root, install and build with `make build-cli` and `cd sysvista-web && npm ci`. Run `make scan TARGET=sysvista-web` to produce `bundles/sysvista-web/` and `bundles/sysvista-web.zip`. Open either in the viewer started by `make dev-web`, or run `node scripts/load-bundle.ts bundles/sysvista-web.zip` to validate it without the UI.

The CLI accepts `scan <root> --output <folder>` and `bundle --input <folder> --archive <zip> [--source-root <root> | --no-source]`. A zip with source verifies source hashes against the scan and the trusted root; omit source with `--no-source`. The folder bundle carries a source index but no embedded source bytes. The zip and folder use schema version 3. The viewer's dev startup loads a committed fixture; importing another bundle replaces it.

## sysvista.toml

Place this optional file at the scan root. Every accepted key is listed below. Unknown keys are rejected. Arrays default to empty unless shown.

```toml
[repository]
name = "team/project"                  # ID fallback if Git origin is absent

[discovery]
include = ["src/**"]                  # extra paths admitted by discovery
exclude = ["**/generated/**"]        # paths excluded from discovery
extra_extensions = { vue = "typescript" } # extension without dot -> language

[viewer]
visible_extensions = ["ts", "tsx", "js", "jsx", "mjs", "cjs", "rs", "py"]

[findings.unresolved]
exclude_reasons = ["dynamic"]         # unresolved-reference reasons ignored for findings

[[modules]]
name = "Frontend"                      # unique; Unassigned is reserved
selectors = ["src/ui/**"]             # globs or paths are accepted aliases
tags = ["ui"]

[[forbidden_dependencies]]
from = "Frontend"                      # source is an accepted alias
to = "Unassigned"                      # target is an accepted alias
```

`include`, `exclude`, and `extra_extensions` may also appear at the TOML root; those values are merged into `[discovery]`. `modules.selectors` also accepts `globs` or `paths`. A module needs a name and at least one selector. Forbidden dependency `from` and `to` values must name configured modules or `Unassigned`. The manifest's `config_snapshot` records the resolved values, including defaults and merged discovery aliases. The viewer extension list controls default hierarchy visibility; hidden files remain inventoried. Excluded unresolved reasons are removed before per-file finding counts are computed.

## Validation

Run `cargo test` in `sysvista-cli`, `npm test` in `sysvista-analyzer` and `sysvista-web`, and `./scripts/regen-schema.sh`. The committed `scripts/merge-gate.sh` runs the integrated release checks. `docs/bundle-format.md` details archive limits and the navigation contract.
