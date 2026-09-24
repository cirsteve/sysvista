# sysvista-cli

Rust CLI for SysVista v2 bundles. From the repository root, run `make build-cli`, then `sysvista-cli/target/release/sysvista-cli scan <root> --output <bundle-directory>`. Use `bundle --input <directory> --archive <zip> --source-root <root>` to include verified source bytes, or `--no-source` to omit them. The old `--format v1` scan remains available for legacy consumers.

`discovery::Config::load` reads `sysvista.toml` and resolves root-level discovery aliases. `scanner::scan_v2` inventories files, records hash/length/line count from one read per included file, runs heuristic analysis and the embedded TypeScript analyzer, then merges results. Function-scoped analyzer declarations carry `is_local`. `hierarchy::derive` builds physical and logical scopes; `output::v2` assigns length-framed IDs and writes canonical graph records. `bundle` writes the folder and validates source hashes before packing a zip. The manifest contains `config_snapshot`, the resolved configuration used for the scan.

Repository identity comes from `git remote get-url origin`, then `[repository] name`, then a hash of the scan path. The analyzer extracts its embedded JavaScript to `$XDG_CACHE_HOME/sysvista` or `$HOME/.cache/sysvista`.

Run `cargo test` in this directory and `../scripts/regen-schema.sh` at the repository root after contract changes. The schema and generated viewer types must be committed together. The web viewer loads schema version 3.
