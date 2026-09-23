#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cargo run --quiet --manifest-path "$repo_root/sysvista-cli/Cargo.toml" -- \
  schema --output "$repo_root/schema/sysvista-v2.schema.json"

"$repo_root/sysvista-web/node_modules/.bin/json2ts" \
  --input "$repo_root/schema/sysvista-v2.schema.json" \
  --output "$repo_root/sysvista-web/src/types/v2.generated.ts"
sed -i '1{/^\/\* eslint-disable \*\/$/d;}' "$repo_root/sysvista-web/src/types/v2.generated.ts"

fixture_bundle="$(mktemp -d)"
trap 'rm -rf "$fixture_bundle"' EXIT
cargo run --quiet --manifest-path "$repo_root/sysvista-cli/Cargo.toml" -- \
  scan "$repo_root/sysvista-cli/tests/fixtures/hierarchy" --output "$fixture_bundle/bundle" >/dev/null
node - "$fixture_bundle/bundle" "$repo_root/sysvista-web/src/test/fixtures" <<'NODE'
const fs = require('node:fs');
const path = require('node:path');
const [bundle, destination] = process.argv.slice(2);
const read = (name) => JSON.parse(fs.readFileSync(path.join(bundle, name), 'utf8'));
const manifest = read('manifest.json');
manifest.scanned_at = '2000-01-01T00:00:00Z';
manifest.root = '/fixture/hierarchy';
const graph = read('graph.json');
const index = read('index/scopes.json');
const snapshot = { ...graph, manifest, diagnostics: read('diagnostics.json'), findings: read('findings.json'), scope_index: index };
const write = (relative, value) => fs.writeFileSync(path.join(destination, relative), JSON.stringify(value, null, 2) + '\n');
write('projection/root-with-three-scopes.json', { snapshot, index });
write('flow/bounded.json', snapshot);
NODE
