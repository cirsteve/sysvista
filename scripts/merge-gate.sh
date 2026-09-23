#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

expect_failure() {
  local expected="$1"; shift
  local output="$work/rejection.log"
  if "$@" >"$output" 2>&1; then
    echo "expected rejection, command succeeded: $*" >&2
    exit 1
  fi
  if ! rg -i "$expected" "$output"; then
    echo "rejection lacked expected text: $expected" >&2
    cat "$output" >&2
    exit 1
  fi
}

npm ci --prefix sysvista-analyzer
npm run build --prefix sysvista-analyzer
npm run typecheck --prefix sysvista-analyzer
npm test --prefix sysvista-analyzer
npm ci --prefix sysvista-web
cargo clean --manifest-path sysvista-cli/Cargo.toml
cargo build --manifest-path sysvista-cli/Cargo.toml
cargo test --manifest-path sysvista-cli/Cargo.toml
npm run build --prefix sysvista-web
npm test --prefix sysvista-web
npm run lint --prefix sysvista-web
./scripts/regen-schema.sh
git diff --exit-code -- schema/sysvista-v2.schema.json sysvista-web/src/types/v2.generated.ts sysvista-web/src/test/fixtures/projection/root-with-three-scopes.json sysvista-web/src/test/fixtures/flow/bounded.json
export XDG_CACHE_HOME="$work/cache"

cli="$repo_root/sysvista-cli/target/debug/sysvista-cli"
for target in sysvista-web .; do
  name="${target//\//_}"
  [[ "$target" != "." ]] || name=root
  source_root="$(realpath "$target")"
  folder="$work/$name-folder"
  "$cli" scan "$source_root" --output "$folder"
  "$cli" bundle --input "$folder" --archive "$work/$name-with-source.zip" --source-root "$source_root"
  "$cli" bundle --input "$folder" --archive "$work/$name-no-source.zip" --no-source
  mkdir "$work/$name-folder-with-source"
  unzip -q "$work/$name-with-source.zip" -d "$work/$name-folder-with-source"
  for bundle in "$folder" "$work/$name-folder-with-source" "$work/$name-with-source.zip" "$work/$name-no-source.zip"; do
    node scripts/load-bundle.ts "$bundle"
  done
done

# Two scan paths with the same origin and identical source must produce the same graph IDs.
mkdir -p "$work/root-a" "$work/root-b"
cp -a sysvista-cli/tests/fixtures/determinism/. "$work/root-a/"
cp -a sysvista-cli/tests/fixtures/determinism/. "$work/root-b/"
origin_url="$(git remote get-url origin)"
for root in "$work/root-a" "$work/root-b"; do
  git init -q "$root"
  git -C "$root" remote add origin "$origin_url"
done
"$cli" scan "$work/root-a" --output "$work/ids-a"
"$cli" scan "$work/root-b" --output "$work/ids-b"
node - "$work/ids-a/graph.json" "$work/ids-b/graph.json" <<'NODE'
const fs = require('node:fs');
const [a, b] = process.argv.slice(2).map(path => JSON.parse(fs.readFileSync(path, 'utf8')));
for (const collection of ['source_files', 'entities', 'relationships', 'modules']) {
  const ids = graph => graph[collection].map(item => item.id).sort();
  if (JSON.stringify(ids(a)) !== JSON.stringify(ids(b))) throw new Error(`non-deterministic ${collection} IDs`);
}
NODE

SYSVISTA_SCALE_SIZES=small scripts/measure-scale.sh

cp "$work/sysvista-web-with-source.zip" "$work/truncated.zip"
truncate -s 100 "$work/truncated.zip"
expect_failure 'zip|invalid|end of central' node scripts/load-bundle.ts "$work/truncated.zip"

cp -a "$work/sysvista-web-folder" "$work/tampered"
node - "$work/tampered/source-index.json" <<'NODE'
const fs = require('node:fs');
const path = process.argv[2];
const index = JSON.parse(fs.readFileSync(path, 'utf8'));
const item = index.files.find(file => file.source_available && file.content_hash);
if (!item) throw new Error('fixture has no available source');
item.content_hash = '0'.repeat(64);
fs.writeFileSync(path, JSON.stringify(index));
NODE
expect_failure 'source changed after scan' "$cli" bundle --input "$work/tampered" --archive "$work/tampered.zip" --source-root "$(realpath sysvista-web)"

cp -a "$work/sysvista-web-folder" "$work/oversized"
node - "$work/oversized" <<'NODE'
const fs = require('node:fs');
const path = require('node:path');
const root = process.argv[2];
fs.writeFileSync(path.join(root, 'oversized.json'), 'x'.repeat(3 * 1024 * 1024));
const manifestPath = path.join(root, 'manifest.json');
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
manifest.files.push('oversized.json');
fs.writeFileSync(manifestPath, JSON.stringify(manifest));
NODE
expect_failure 'cap|too large' "$cli" bundle --input "$work/oversized" --archive "$work/oversized.zip" --no-source

echo 'merge gate passed: builds, tests, schema, four bundle variants per root, IDs, scale and rejections'
