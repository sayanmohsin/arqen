#!/usr/bin/env bash

set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
nice_code_repo=https://github.com/sayanmohsin/nice-code.git

if [[ -n "${NICE_CODE_DIR:-}" ]]; then
  nice_code_dir=$NICE_CODE_DIR
else
  nice_code_dir=${NICE_CODE_CACHE_DIR:-"${TMPDIR:-/tmp}/arqen-nice-code"}
  if [[ -d "$nice_code_dir/.git" ]]; then
    git -C "$nice_code_dir" pull --ff-only origin main
  else
    mkdir -p "$(dirname "$nice_code_dir")"
    git clone --depth 1 --branch main "$nice_code_repo" "$nice_code_dir"
  fi
fi

if [[ ! -f "$nice_code_dir/scripts/nice-code.mjs" ]]; then
  printf 'nice-code checker not found at %s\n' "$nice_code_dir" >&2
  printf 'Set NICE_CODE_DIR to an existing Nice Code checkout or rerun with network access.\n' >&2
  exit 1
fi

node "$nice_code_dir/scripts/nice-code.mjs" \
  --project "$root_dir" \
  --all \
  --format agent \
  --include-review \
  --max-findings 100
