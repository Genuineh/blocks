#!/usr/bin/env sh
set -eu

script_dir="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
block_root="$(dirname "$script_dir")"
repo_root="$(CDPATH= cd -- "$block_root/../.." && pwd)"

output="$(cargo run -p blocks-cli -- run "$repo_root/blocks" demo.echo "$script_dir/success.input.json")"
printf '%s\n' "$output" | grep '"text": "hello from example"' >/dev/null
