#!/usr/bin/env bash
#
# Detect and configure a Homebrew LLVM toolchain compatible with the local rustc.
#
# The thingd-native feature pulls in:  thingd → rocksdb → librocksdb-sys → bindgen → clang-sys
# librocksdb-sys builds C++ code and dynamically links libclang.dylib at build time.
# On macOS, the linker must resolve a libclang whose LLVM symbols match rustc's bundled LLVM.
#
# Usage:
#   source scripts/setup-llvm.sh        # sets env vars in the current shell
#
# The script:
#   1. Skips on Linux (CI/Docker use apt-managed packages).
#   2. Reads the LLVM major version from `rustc --version --verbose`.
#   3. Searches Homebrew for a matching `llvm@<MAJOR>` formula.
#   4. Sets LIBCLANG_PATH, LLVM_CONFIG_PATH, and DYLD_LIBRARY_PATH.
#   5. Fails with a clear remediation message when no compatible toolchain exists.

set -euo pipefail

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

die() {
  printf '\033[1;31merror:\033[0m %s\n' "$1" >&2
  exit 1
}

info() {
  printf '\033[1;34mllvm-setup:\033[0m %s\n' "$1" >&2
}

warn() {
  printf '\033[1;33mllvm-setup:\033[0m %s\n' "$1" >&2
}

# ---------------------------------------------------------------------------
# Skip on Linux — CI and Docker handle LLVM via apt.
# ---------------------------------------------------------------------------

if [[ "$(uname -s)" != "Darwin" ]]; then
  info "non-macOS detected; skipping Homebrew LLVM detection"
  exit 0
fi

# ---------------------------------------------------------------------------
# Skip if already correctly configured.
# ---------------------------------------------------------------------------

if [[ -n "${LIBCLANG_PATH:-}" ]]; then
  if [[ -f "$LIBCLANG_PATH/libclang.dylib" ]] || [[ -f "$LIBCLANG_PATH/libclang.so" ]]; then
    info "LIBCLANG_PATH is already set and valid: $LIBCLANG_PATH"
    exit 0
  fi
  warn "LIBCLANG_PATH is set but invalid ($LIBCLANG_PATH/libclang.dylib not found); re-detecting"
fi

# ---------------------------------------------------------------------------
# Check for Homebrew.
# ---------------------------------------------------------------------------

if ! command -v brew >/dev/null 2>&1; then
  die "Homebrew is not installed. Install it from https://brew.sh or set LIBCLANG_PATH manually."
fi

# ---------------------------------------------------------------------------
# Detect the LLVM major version used by rustc.
# ---------------------------------------------------------------------------

if ! command -v rustc >/dev/null 2>&1; then
  die "rustc is not installed. Install Rust via https://rustup.rs"
fi

LLVM_VERSION_RAW=$(rustc --version --verbose 2>/dev/null | grep -i "LLVM version" | awk '{print $NF}')
if [[ -z "$LLVM_VERSION_RAW" ]]; then
  die "Could not detect LLVM version from rustc. Run: rustc --version --verbose"
fi

LLVM_MAJOR=${LLVM_VERSION_RAW%%.*}
if [[ -z "$LLVM_MAJOR" || ! "$LLVM_MAJOR" =~ ^[0-9]+$ ]]; then
  die "Parsed invalid LLVM major version from '$LLVM_VERSION_RAW'"
fi

info "rustc uses LLVM $LLVM_VERSION_RAW (major: $LLVM_MAJOR)"

# ---------------------------------------------------------------------------
# Search Homebrew for a matching llvm@<MAJOR> formula.
# We try the exact major first, then walk downwards to avoid version mixing.
# ---------------------------------------------------------------------------

MATCHED_PREFIX=""

for candidate in "llvm@$LLVM_MAJOR" llvm; do
  candidate_prefix="$(brew --prefix "$candidate" 2>/dev/null || true)"
  if [[ -z "$candidate_prefix" ]]; then
    continue
  fi

  # Verify libclang.dylib exists (the build script links it at runtime).
  if [[ -f "$candidate_prefix/lib/libclang.dylib" ]]; then
    # Verify the LLVM major version matches what rustc expects.
    candidate_version="$("$candidate_prefix/bin/llvm-config" --version 2>/dev/null || true)"
    candidate_major="${candidate_version%%.*}"

    if [[ "$candidate_major" == "$LLVM_MAJOR" ]]; then
      MATCHED_PREFIX="$candidate_prefix"
      break
    else
      warn "skipping $candidate ($candidate_version) — major version $candidate_major != rustc's $LLVM_MAJOR"
    fi
  fi
done

# Also check Cellar directly for versioned formulas Homebrew might not expose.
if [[ -z "$MATCHED_PREFIX" ]]; then
  for cellar_dir in /opt/homebrew/Cellar/llvm@"$LLVM_MAJOR"/*/; do
    if [[ -d "$cellar_dir" && -f "$cellar_dir/lib/libclang.dylib" ]]; then
      MATCHED_PREFIX="$cellar_dir"
      break
    fi
  done
fi

# ---------------------------------------------------------------------------
# Set environment variables or fail with remediation.
# ---------------------------------------------------------------------------

if [[ -z "$MATCHED_PREFIX" ]]; then
  cat >&2 <<EOF

\033[1;31merror: no compatible LLVM found\033[0m

rustc $(rustc --version 2>/dev/null || echo "unknown") uses LLVM $LLVM_VERSION_RAW,
but no matching Homebrew formula was found.

Install the matching formula:

    brew install llvm@$LLVM_MAJOR

Then re-run this script:

    source scripts/setup-llvm.sh

Or set the variables manually:

    export LIBCLANG_PATH="\$(brew --prefix llvm@$LLVM_MAJOR)/lib"
    export LLVM_CONFIG_PATH="\$(brew --prefix llvm@$LLVM_MAJOR)/bin/llvm-config"
    export DYLD_LIBRARY_PATH="\$(brew --prefix llvm@$LLVM_MAJOR)/lib"

EOF
  exit 1
fi

export LIBCLANG_PATH="$MATCHED_PREFIX/lib"
export LLVM_CONFIG_PATH="$MATCHED_PREFIX/bin/llvm-config"
export DYLD_LIBRARY_PATH="${DYLD_LIBRARY_PATH:+$DYLD_LIBRARY_PATH:}$MATCHED_PREFIX/lib"

info "configured LLVM $LLVM_VERSION_RAW from $MATCHED_PREFIX"
info "  LIBCLANG_PATH=$LIBCLANG_PATH"
info "  LLVM_CONFIG_PATH=$LLVM_CONFIG_PATH"
info "  DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH"
