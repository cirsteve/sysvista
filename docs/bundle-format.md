# SysVista bundle format

A v2 directory bundle contains:

- `manifest.json`: schema/tool metadata, inventory and validation summary, the canonical file list, and `source_included`.
- `graph.json`: entities, relationships, modules, projections, unresolved references, evidence, claims, payload contracts, and forbidden-dependency rules.
- `diagnostics.json` and `findings.json`: canonical, independently loadable post-analysis output.
- `config.snapshot.toml`: the logical-module selectors, tags, and forbidden-dependency rules used by the scan.
- `index/scopes.json`: navigation slices for every physical and logical projection scope.
- `source-index.json`: each `FileId`, normalized path, SHA-256 content hash, byte length, and source availability.

`sysvista bundle --input <directory> --archive <path>` writes a zip. Unless `--no-source` is supplied, available source bytes are stored once at `source/<sha256>` and verified against `source-index.json`. `--no-source` omits all `source/` entries and sets `manifest.source_included` to `false` in the archive.

All entry names are relative normalized paths. Writers reject absolute paths and any `..` component with a typed `UnsafePath` error.

The default source-entry cap is 2 MiB. Oversized source is omitted, remains present in `source-index.json` with `source_available: false`, and produces a `source_unavailable` diagnostic. The default total uncompressed archive cap is 512 MiB. Source files are re-hashed while packing so a changed file cannot be archived under a stale hash.
