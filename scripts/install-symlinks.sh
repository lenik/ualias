#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Invoked as:
#   bash scripts/install-symlinks.sh <SOURCE_ROOT> <BUILD_ROOT> <prefix> <bindir> <datadir> <localedir> <mandir>
set -euo pipefail
SOURCE_ROOT="${1:?}"
BUILD_ROOT="${2:?}"
prefix="${3:?}"
bindir="${4:?}"
datadir="${5:?}"
localedir="${6:?}"
mandir="${7:?}"

mkdir -p "$bindir" "$datadir/bash-completion/completions" "$localedir" "$mandir/man1"

sudo ln -sfn "${BUILD_ROOT}/ualias" "$bindir/ualias"
sudo ln -sfn "${BUILD_ROOT}/ualias.1" "$mandir/man1/ualias.1"
sudo ln -sfn "${SOURCE_ROOT}/ualias.bash" "$datadir/bash-completion/completions/ualias"

printf "Symlinks installed under %s\n" "$prefix"
