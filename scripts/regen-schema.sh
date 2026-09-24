#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cargo run --quiet --manifest-path "$repo_root/sysvista-cli/Cargo.toml" -- \
  schema --output "$repo_root/schema/sysvista-v2.schema.json"

"$repo_root/sysvista-web/node_modules/.bin/json2ts" \
  --input "$repo_root/schema/sysvista-v2.schema.json" \
  --output "$repo_root/sysvista-web/src/types/v2.generated.ts"
