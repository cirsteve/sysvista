# SysVista

SysVista scans a repository into a version 3 folder bundle. The React viewer opens the folder or its zip archive and shows physical and logical scopes, relationships, findings, flow, and verified source when included.

## Build and scan

Requires Rust 1.85+ and Node 22+.

```sh
make build-cli
cd sysvista-web && npm ci && cd ..
make scan TARGET=sysvista-web
```

`make scan` writes `bundles/sysvista-web/` and `bundles/sysvista-web.zip`, including source bytes in the zip. For an explicit scan and a zip without source:

```sh
cargo run --manifest-path sysvista-cli/Cargo.toml -- scan /path/to/repo --output bundles/repo
cargo run --manifest-path sysvista-cli/Cargo.toml -- bundle --input bundles/repo --archive bundles/repo-no-source.zip --no-source
```

Run `make dev-web`, then use **Import** for a zip or **Bundle folder** for a directory bundle. Drag and drop also works. The development viewer initially loads a committed test fixture. See [USAGE.md](USAGE.md) for the complete configuration reference and bundle commands.

## Contract

A folder bundle contains `manifest.json` (including the resolved `config_snapshot`), `graph.json`, `diagnostics.json`, `findings.json`, `source-index.json`, and `index/scopes.json`. The CLI generates [schema/sysvista-v2.schema.json](schema/sysvista-v2.schema.json); `scripts/regen-schema.sh` refreshes the schema, generated web types, and committed fixtures. `scripts/load-bundle.ts <folder-or-zip>` checks a bundle with the viewer's schema and reference validators.

TypeScript and JavaScript use an embedded compiler-based analyzer. Other supported languages also receive heuristic detection. Function-scoped declarations remain in the graph with `is_local: true` and are hidden in the viewer's default projection. Repository identity uses the normalized Git origin URL when present, then a configured name, then a scan-path hash.

The legacy `scan --format v1` option remains for older consumers. New integrations should use the version 3 bundle.

## Configuration keys

`sysvista.toml` accepts the following keys. Unknown keys fail loading. The resolved values appear in `manifest.config_snapshot`.

| Key | Purpose |
| --- | --- |
| `repository.name` | Repository ID fallback when no Git origin exists. |
| `discovery.include`, `discovery.exclude` | Discovery path globs. |
| `discovery.extra_extensions` | Map an extension without a dot to a language. |
| Root `include`, `exclude`, `extra_extensions` | Aliases merged into the corresponding discovery keys. |
| `modules[].name`, `modules[].selectors`, `modules[].tags` | Logical module name, membership globs and labels. `globs` and `paths` alias `selectors`. |
| `forbidden_dependencies[].from`, `.to` | Exact source and target module names. `source` and `target` are accepted aliases. |
| `viewer.visible_extensions` | Extensions shown in the default hierarchy; defaults to `ts`, `tsx`, `js`, `jsx`, `mjs`, `cjs`, `rs`, `py`. |
| `findings.unresolved.exclude_reasons` | Unresolved reasons omitted before finding counts. |
