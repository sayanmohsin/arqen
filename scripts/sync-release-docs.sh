#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 || ! "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  printf 'usage: %s VERSION\n' "$0" >&2
  exit 2
fi

version=$1
series=${version%.*}
export RELEASE_VERSION="$version" RELEASE_SERIES="$series"

# Release Please owns the crate version and changelog. This script keeps the
# workspace metadata and public release examples in sync after a release PR
# is created, so post-merge audits validate the same contract users see.
perl -0pi -e 's/^(version = ")[^"]+("\s*)$/${1}$ENV{RELEASE_VERSION}$2/m' Cargo.toml
perl -0pi -e 's/arqen = "[0-9]+\.[0-9]+"/arqen = "$ENV{RELEASE_SERIES}"/' README.md
perl -0pi -e 's/(current published release is \*\*)[^*]+(\*\*)/${1}$ENV{RELEASE_VERSION}$2/' CHANGELOG.md
perl -0pi -e 's/current Arqen [0-9]+\.[0-9]+ release/current Arqen $ENV{RELEASE_SERIES} release/' docs/getting-started.md
perl -0pi -e 's/(arqen = .*version = ")[0-9]+\.[0-9]+(")/${1}$ENV{RELEASE_SERIES}$2/' docs/testing.md docs/troubleshooting.md

workspace_version=$(sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | sed -n 's/^version = "\([^"]*\)"/\1/p' | head -n 1)

if [[ "$workspace_version" != "$version" ]] || \
   ! rg -q "arqen = \"${series}\"" README.md || \
   ! rg -q "current published release is \\*\\*${version}\\*\\*" CHANGELOG.md || \
   ! rg -q "current Arqen ${series} release" docs/getting-started.md || \
   ! rg -q "arqen = .*version = \"${series}\"" docs/testing.md docs/troubleshooting.md; then
  printf 'release docs sync: expected release references were not updated for %s\n' "$version" >&2
  exit 1
fi

printf 'release docs sync: updated public metadata for Arqen %s\n' "$version"
