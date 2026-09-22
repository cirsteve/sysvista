# Analyzer packaging

The TypeScript analyzer is bundled by esbuild into one `dist/analyzer.js`. The Rust build copies that file into `OUT_DIR`, embeds it with `include_bytes!`, and at runtime extracts it to `$XDG_CACHE_HOME/sysvista/analyzer-<version>.js` (or the system temporary directory when XDG is unset). It invokes `node` from `PATH`; it never installs packages or runs repository scripts at scan time.

The completed analyzer bundle, including TypeScript's standard-library declarations, is 13,168,154 bytes. In the local debug measurement the CLI grew from the pre-analyzer 56,429,000 bytes to 71,243,176 bytes, a 14,814,176-byte delta (debug metadata makes that larger than the embedded payload). Release/platform measurements should be refreshed when TypeScript or esbuild changes.

Before analysis the CLI runs `--handshake` and requires integer contract version 1. Missing Node, a bad bundle, and contract mismatch become error diagnostics and skip only the analyzer stage; heuristic output remains available. The packaging fallback, if embedding becomes unworkable on a target, is a sidecar bundle selected by a future `--analyzer <path>` option. The CLI is not replaced and no runtime npm install is permitted.
