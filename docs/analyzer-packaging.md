# Analyzer packaging

Install analyzer dependencies with `npm ci` in `sysvista-analyzer`. The CLI build script runs `npm run build` when those dependencies are installed, copies `dist/analyzer.js` into Cargo's `OUT_DIR`, and embeds that bundle in the CLI binary. A missing dependency install requires an existing dist bundle; otherwise the build fails. The build does not install packages.

At build time, npm's cache is rooted at `$XDG_CACHE_HOME/sysvista/npm`, then `$HOME/.cache/sysvista/npm`; the operating system temporary directory is the last fallback. At scan time the CLI uses the same base directory for its embedded bundle, verifies analyzer contract version 2 through a Node handshake, and extracts the bundle into the private cache. The extracted file is checked before use and atomically replaced when its bytes differ. Node must be on `PATH`. Analyzer failures become diagnostics while the heuristic stage remains available.

The bundle contains TypeScript's compiler and standard libraries, so it increases binary size. The analyzer output includes declarations, relationships and an `is_local` flag for function-scoped declarations; the CLI preserves that flag on entities. There is no runtime npm install or sidecar dependency.
