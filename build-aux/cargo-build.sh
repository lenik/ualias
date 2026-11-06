#!/bin/sh
# Install the release ualias binary to the path given as $1. Meson provides the
# final install path; we always build to ${MESON_BUILD_DIR}/cargo-target.
set -euf
out="$1"
root="$2"
b="$3"
# Meson passes @OUTPUT@ as a bare name; resolve it under the build dir before
# we cd into $root for cargo.
case "$out" in
  /*) ;;
  *) out="${b}/${out}" ;;
esac
export CARGO_TARGET_DIR="${b}/cargo-target"
# Avoid broken sccache in some environments; Meson can override via env in tests.
[ "${UALIAS_CARGO_ENABLE_SCCACHE:-0}" = 1 ] || unset RUSTC_WRAPPER
cd "$root"
cargo build --release --locked --bin ualias
install -m755 "${CARGO_TARGET_DIR}/release/ualias" "${out}.tmp"
mv -f "${out}.tmp" "${out}"
