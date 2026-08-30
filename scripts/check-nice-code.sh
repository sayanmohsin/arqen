#!/usr/bin/env bash

set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
nice_code_dir=${NICE_CODE_DIR:-"$root_dir/../nice-code"}

if [[ ! -f "$nice_code_dir/scripts/nice-code.mjs" ]]; then
  printf 'nice-code checker not found at %s\n' "$nice_code_dir" >&2
  printf 'Set NICE_CODE_DIR to a checkout of https://github.com/sayanmohsin/nice-code.git\n' >&2
  exit 1
fi

node "$nice_code_dir/scripts/nice-code.mjs" \
  --project "$root_dir" \
  --all \
  --format agent \
  --include-review \
  --max-findings 100
