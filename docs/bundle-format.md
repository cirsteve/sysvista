# SysVista bundle format

A v2 directory bundle contains:

- `manifest.json`: schema/tool metadata, inventory and validation summary, the canonical file list, and `source_included`.
- `graph.json`: entities, relationships, modules, projections, unresolved references, evidence, claims, payload contracts, findings, and forbidden-dependency rules. Findings are also emitted separately for direct post-analysis consumption.
- `diagnostics.json` and `findings.json`: canonical, independently loadable post-analysis output.
- `config.snapshot.toml`: the logical-module selectors, tags, and forbidden-dependency rules used by the scan.
- `index/scopes.json`: navigation slices for every physical and logical projection scope.
- `source-index.json`: each `FileId`, normalized path, SHA-256 content hash, byte length, and source availability.

`sysvista bundle --input <directory> --archive <path>` writes a zip. Unless `--no-source` is supplied, available source bytes are stored once at `source/<sha256>` and verified against `source-index.json`. The source root defaults to the current directory and can be supplied explicitly with `--source-root <path>`; it must match the scanned root in the manifest, and every resolved source must remain inside it. `--no-source` omits all `source/` entries and sets `manifest.source_included` to `false` in the archive.

All entry names are portable, relative, normalized `/`-separated paths. Writers reject absolute paths, drive prefixes, backslashes, empty components, and any `.` or `..` component with a typed `UnsafePath` error.

The source index contains every discovered `FileId`; hash and byte-length fields are absent when those values could not be captured, and `source_available` is then false. The default source-entry cap is 2 MiB. Oversized source is omitted, remains present in `source-index.json` with `source_available: false`, and produces a `source_unavailable` diagnostic. The default total uncompressed archive cap is 512 MiB. Source files are re-hashed and length-checked before applying the archive entry cap, so changed files cannot be silently omitted or archived under stale metadata.
