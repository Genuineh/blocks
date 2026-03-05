#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> cargo test (root workspace)"
cargo test --manifest-path "$repo_root/Cargo.toml"

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
