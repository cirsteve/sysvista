#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
seed="${SYSVISTA_SCALE_SEED:-20260922}"
output_root="${SYSVISTA_SCALE_OUTPUT:-$(mktemp -d)}"
sizes="${SYSVISTA_SCALE_SIZES:-10000 100000 250000}"
analyzer_heap_mb="${SYSVISTA_ANALYZER_HEAP_MB:-8192}"
cli="$repository_root/sysvista-cli/target/release/sysvista-cli"

mkdir -p "$output_root"
cargo build --release --manifest-path "$repository_root/sysvista-cli/Cargo.toml" >/dev/null

echo "Reference machine"
echo "- Hostname: $(hostname)"
echo "- CPU: $(lscpu | awk -F: '/Model name/{sub(/^[[:space:]]+/, "", $2); print $2; exit}')"
echo "- RAM: $(awk '/MemTotal/{printf "%.1f GiB", $2 / 1024 / 1024}' /proc/meminfo)"
echo "- Date: $(date +%F)"
echo "- Seed: $seed"
echo "- Analyzer heap: $analyzer_heap_mb MiB"
echo
echo "| Relationships | CLI scan (ms) | Bundle bytes | Warm projection (ms) |"
echo "| ---: | ---: | ---: | ---: |"

for relationship_count in $sizes; do
  fixture_root="$output_root/$relationship_count"
  node "$repository_root/scripts/gen-scale-fixture.ts" \
    --relationships "$relationship_count" \
    --seed "$seed" \
    --output "$fixture_root" >/dev/null

  scan_start="$(date +%s%3N)"
  scan_log="$fixture_root/scan.log"
  if ! NODE_OPTIONS="--max-old-space-size=$analyzer_heap_mb ${NODE_OPTIONS:-}" \
    "$cli" scan "$fixture_root/source" --output "$fixture_root/bundle" >"$scan_log" 2>&1; then
    cat "$scan_log" >&2
    exit 1
  fi
  scan_end="$(date +%s%3N)"
  scan_ms="$((scan_end - scan_start))"
  printf '%s\n' "$scan_ms" >"$fixture_root/scan-ms.txt"
  scanned_relationships="$(jq '.relationships | length' "$fixture_root/bundle/graph.json")"
  if [[ "$scanned_relationships" != "$relationship_count" ]]; then
    echo "error: requested $relationship_count relationships but CLI emitted $scanned_relationships" >&2
    cat "$fixture_root/bundle/diagnostics.json" >&2
    exit 1
  fi
  bundle_bytes="$(du -sb "$fixture_root/bundle" | awk '{print $1}')"

  if ! benchmark_output="$(
    cd "$repository_root/sysvista-web"
    SYSVISTA_BENCH_FIXTURE="$fixture_root/fixture.json" \
      SYSVISTA_BENCH_RELATIONSHIPS="$relationship_count" \
      SYSVISTA_RUN_PROJECTION_BENCHMARK=1 \
      npx vitest run src/lib/projection/project.test.ts --configLoader runner --maxWorkers=1 2>&1
  )"; then
    printf '%s\n' "$benchmark_output" >&2
    exit 1
  fi
  projection_ms="$(printf '%s\n' "$benchmark_output" | sed -n 's/.*benchmark: \([0-9][0-9.]*\)ms.*/\1/p' | tail -1)"
  if [[ -z "$projection_ms" ]]; then
    printf '%s\n' "$benchmark_output" >&2
    echo "error: projection benchmark did not report a measurement" >&2
    exit 1
  fi

  echo "| $relationship_count | $scan_ms | $bundle_bytes | $projection_ms |"
done

echo
echo "Generated fixtures remain in $output_root and are not repository artifacts."
