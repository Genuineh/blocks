#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bcl_gate_mode="${BLOCKS_BCL_GATE_MODE:-warn}"
prefetch_frontend_hosts="${BLOCKS_REPO_CHECK_PREFETCH_FRONTEND_HOSTS:-}"

if [[ -z "$prefetch_frontend_hosts" && "${CI:-}" == "true" ]]; then
  prefetch_frontend_hosts="1"
fi

echo "==> cargo test (root workspace)"
cargo test --manifest-path "$repo_root/Cargo.toml"

if [[ "$prefetch_frontend_hosts" == "1" ]]; then
  echo "==> prefetch frontend host dependencies for offline probes"
  cargo fetch \
    --locked \
    --manifest-path "$repo_root/mocs/counter-panel-web/src-tauri/Cargo.toml"
  cargo fetch \
    --locked \
    --manifest-path "$repo_root/mocs/greeting-panel-web/src-tauri/Cargo.toml"
fi

echo "==> demo.echo block conformance"
cargo run -p blocks-cli -- \
  conformance run block "$repo_root/blocks/demo.echo"

echo "==> echo-pipeline moc conformance"
cargo run -p blocks-cli -- \
  conformance run moc "$repo_root/blocks" "$repo_root/mocs/echo-pipeline/moc.yaml"

echo "==> BCL conformance (echo-pipeline, gate=$bcl_gate_mode)"
cargo run -p blocks-cli -- \
  conformance run bcl "$repo_root/blocks" "$repo_root/mocs/echo-pipeline/moc.bcl" \
  --check-against "$repo_root/mocs/echo-pipeline/moc.yaml" \
  --gate-mode "$bcl_gate_mode"

echo "==> BCL conformance (greeting-panel-web, gate=$bcl_gate_mode)"
cargo run -p blocks-cli -- \
  conformance run bcl "$repo_root/blocks" "$repo_root/mocs/greeting-panel-web/moc.bcl" \
  --check-against "$repo_root/mocs/greeting-panel-web/moc.yaml" \
  --gate-mode "$bcl_gate_mode"

echo "==> blocks catalog export"
cargo run -p blocks-cli -- \
  catalog export "$repo_root/blocks" --json >/dev/null

echo "==> demo.echo block doctor"
cargo run -p blocks-cli -- \
  block doctor "$repo_root/blocks" "$repo_root/blocks/demo.echo" --json >/dev/null

echo "==> echo-pipeline moc doctor"
cargo run -p blocks-cli -- \
  moc doctor "$repo_root/blocks" "$repo_root/mocs/echo-pipeline/moc.yaml" --json >/dev/null

echo "==> echo-pipeline BCL graph"
cargo run -p blocks-cli -- \
  moc bcl graph "$repo_root/blocks" "$repo_root/mocs/echo-pipeline/moc.bcl" --json >/dev/null

echo "==> demo.echo compat check"
cargo run -p blocks-cli -- \
  compat block "$repo_root/blocks/demo.echo/block.yaml" "$repo_root/blocks/demo.echo/block.yaml" --json >/dev/null

echo "==> demo.echo upgrade preview"
cargo run -p blocks-cli -- \
  upgrade block "$repo_root/blocks/demo.echo" --json >/dev/null

echo "==> counter-panel-web host headless probe"
cargo --offline run \
  --manifest-path "$repo_root/mocs/counter-panel-web/src-tauri/Cargo.toml" \
  -- \
  --headless-probe

echo "==> greeting-panel-web state logic"
node --test "$repo_root/mocs/greeting-panel-web/tests/greeting_panel.test.mjs"

echo "==> greeting-panel-web host headless probe"
cargo --offline run \
  --manifest-path "$repo_root/mocs/greeting-panel-web/src-tauri/Cargo.toml" \
  -- \
  --headless-probe
