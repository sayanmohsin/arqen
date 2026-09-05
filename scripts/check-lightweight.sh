#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
for features in default http-client cli; do
  args=()
  if [[ "$features" != default ]]; then
    args=(--no-default-features --features "$features")
  fi
  graph=$(cargo tree --locked -p arqen -e normal,build --prefix none "${args[@]}")
  if printf '%s\n' "$graph" | rg '^(thingd|rocksdb|librocksdb-sys|bindgen) '; then
    echo "Native dependency leaked into $features" >&2
    exit 1
  fi
done
