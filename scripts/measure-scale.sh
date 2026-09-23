#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
seed="${SYSVISTA_SCALE_SEED:-20260922}"
output_root="${SYSVISTA_SCALE_OUTPUT:-$(mktemp -d)}"
sizes="${SYSVISTA_SCALE_SIZES:-small}"
analyzer_heap_mb="${SYSVISTA_ANALYZER_HEAP_MB:-8192}"
viewer_heap_mb="${SYSVISTA_VIEWER_HEAP_MB:-8192}"
cli="$repository_root/sysvista-cli/target/release/sysvista-cli"

mkdir -p "$output_root"
cargo build --release --manifest-path "$repository_root/sysvista-cli/Cargo.toml" >/dev/null

for size in $sizes; do
  case "$size" in
    small) files=100; entities=1000; relationships=2500 ;;
    large) files=10000; entities=100000; relationships=250000 ;;
    *) echo "Unknown scale size: $size" >&2; exit 2 ;;
  esac
  fixture_root="$output_root/$size"
  node "$repository_root/scripts/gen-scale-fixture.ts" \
    --files "$files" --entities "$entities" --relationships "$relationships" \
    --seed "$seed" --output "$fixture_root" >/dev/null

  scan_start="$(date +%s%3N)"
  if ! NODE_OPTIONS="--max-old-space-size=$analyzer_heap_mb ${NODE_OPTIONS:-}" \
    "$cli" scan "$fixture_root/source" --output "$fixture_root/bundle" >"$fixture_root/scan.log" 2>&1; then
    cat "$fixture_root/scan.log" >&2
    exit 1
  fi
  scan_end="$(date +%s%3N)"
  NODE_OPTIONS="--max-old-space-size=$viewer_heap_mb ${NODE_OPTIONS:-}" \
    node "$repository_root/scripts/measure-viewer.ts" "$fixture_root/bundle" "$fixture_root/viewer.json"
  bundle_bytes="$(du -sb "$fixture_root/bundle" | awk '{print $1}')"
  node - "$fixture_root/viewer.json" "$fixture_root/report.json" "$size" "$seed" "$((scan_end-scan_start))" "$bundle_bytes" <<'NODE'
import { readFileSync, writeFileSync } from "node:fs";
const [viewerPath, reportPath, size, seed, scanMs, bundleBytes] = process.argv.slice(2);
const viewer = JSON.parse(readFileSync(viewerPath, "utf8"));
const report = { size, seed: Number(seed), scan_ms: Number(scanMs), bundle_bytes: Number(bundleBytes), ...viewer };
writeFileSync(reportPath, JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify(report));
NODE
done

echo "Reports and generated fixtures: $output_root"
