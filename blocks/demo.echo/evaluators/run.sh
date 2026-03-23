#!/usr/bin/env sh
set -eu

script_dir="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
block_root="$(dirname "$script_dir")"

cargo test \
  --manifest-path "$block_root/rust/Cargo.toml" \
  quality_gate_fixture_echo_invariance \
  -- \
  --exact
