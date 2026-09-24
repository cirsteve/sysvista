#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ ! -f "$repository_root/sysvista-analyzer/dist/analyzer.js" ]]; then
  echo "error: analyzer bundle missing; run 'npm ci && npm run build' in sysvista-analyzer" >&2
  exit 2
fi

echo "Validating labeled corpus with the shared snapshot validator"
cargo test \
  --manifest-path "$repository_root/sysvista-cli/Cargo.toml" \
  --test corpus \
  -- \
  --nocapture
