#!/usr/bin/env bash

set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
nice_code_repo=https://github.com/sayanmohsin/nice-code.git
nice_code_version=${NICE_CODE_VERSION:-0.3.2}
nice_code_ref="v${nice_code_version#v}"

if [[ -n "${NICE_CODE_DIR:-}" ]]; then
  nice_code_dir=$NICE_CODE_DIR
else
  nice_code_dir=${NICE_CODE_CACHE_DIR:-"${TMPDIR:-/tmp}/arqen-nice-code-${nice_code_version#v}"}
  if [[ -d "$nice_code_dir/.git" ]]; then
    git -C "$nice_code_dir" fetch --depth 1 origin "refs/tags/$nice_code_ref"
    git -C "$nice_code_dir" checkout --detach "$nice_code_ref"
  else
    mkdir -p "$(dirname "$nice_code_dir")"
    git clone --depth 1 --branch "$nice_code_ref" "$nice_code_repo" "$nice_code_dir"
  fi
fi

if [[ ! -f "$nice_code_dir/scripts/nice-code.mjs" ]]; then
  printf 'nice-code checker not found at %s\n' "$nice_code_dir" >&2
  printf 'Set NICE_CODE_DIR to an existing Nice Code checkout or rerun with network access.\n' >&2
  exit 1
fi

scan_args=(
  --project "$root_dir"
  --format agent
  --include-review
  --max-findings 100
  --ci
  --cache
)
if [[ $# -eq 0 ]]; then
  scan_args+=(--all)
else
  scan_args+=("$@")
fi

node "$nice_code_dir/scripts/nice-code.mjs" "${scan_args[@]}"
