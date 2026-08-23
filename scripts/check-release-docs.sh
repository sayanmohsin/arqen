#!/usr/bin/env bash

set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root_dir"

failures=0

fail() {
  printf 'release docs audit: %s\n' "$1" >&2
  failures=$((failures + 1))
}

crate_version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' crates/arqen/Cargo.toml | head -n 1)
workspace_version=$(sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | sed -n 's/^version = "\([^"]*\)"/\1/p' | head -n 1)
release_version=$(jq -r '."crates/arqen"' .release-please-manifest.json)
thingd_range=$(sed -n 's/.*thingd = { version = "\([^"]*\)".*/\1/p' crates/arqen/Cargo.toml | head -n 1)
crate_series=${crate_version%.*}

if [[ -z "$crate_version" || -z "$workspace_version" || -z "$release_version" || "$release_version" == "null" || -z "$thingd_range" ]]; then
  fail "could not derive all release metadata from Cargo and Release Please files"
fi

if [[ "$crate_version" != "$workspace_version" ]]; then
  fail "workspace version ($workspace_version) does not match crate version ($crate_version)"
fi

if [[ "$crate_version" != "$release_version" ]]; then
  fail "Release Please version ($release_version) does not match crate version ($crate_version)"
fi

if ! rg -q "arqen = \"${crate_series}\"" README.md; then
  fail "README dependency example does not use Arqen series ${crate_series}"
fi

if ! rg -q "current published release is \*\*${crate_version}\*\*" CHANGELOG.md; then
  fail "root CHANGELOG current release does not match ${crate_version}"
fi

if ! rg -q "current Arqen ${crate_series} release" docs/getting-started.md; then
  fail "getting-started release reference does not use Arqen series ${crate_series}"
fi

for example_file in docs/testing.md docs/troubleshooting.md; do
  if ! rg -q "arqen = .*version = \"${crate_series}\"" "$example_file"; then
    fail "$example_file dependency example does not use Arqen series ${crate_series}"
  fi
done

if ! rg -q '<CurrentVersion kind="native-thingd"' docs/architecture.md docs/feature-status.md docs/adapter-contract.md docs/api-stability.md docs/thingd-integration.md; then
  fail "compatibility documentation is not using the dynamic native Thingd metadata component"
fi

while IFS= read -r reference; do
  [[ "$reference" == *"$thingd_range"* ]] || fail "stale Thingd compatibility range: $reference"
done < <(rg -n -o '>=([0-9]+\.[0-9]+\.[0-9]+), <([0-9]+\.[0-9]+\.[0-9]+)' README.md docs --glob '!docs/.vitepress/dist/**' || true)

if git grep -niE 'fjall|fjall[[:space:]_-]*migration|migration[[:space:]_-]*fjall' -- .; then
  fail "forbidden legacy Fjall terminology is present in tracked files"
fi

if ! rg -q 'thingd-migration = ' crates/arqen/Cargo.toml; then
  fail "thingd-migration feature is missing"
fi

if ! rg -q 'NativeToHttpMigrator' crates/arqen/src/lib.rs crates/arqen/src/migration.rs; then
  fail "native-to-HTTP migration API is missing"
fi

if ! rg -q 'ThingdCommand::Migrate|thingd migrate' crates/arqen/src/cli; then
  fail "thingd migration CLI is missing"
fi

if (( failures > 0 )); then
  printf 'release docs audit: %d failure(s)\n' "$failures" >&2
  exit 1
fi

printf 'release docs audit: passed (Arqen %s, Thingd %s)\n' "$crate_version" "$thingd_range"
