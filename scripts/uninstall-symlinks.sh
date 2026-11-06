#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Invoked as:
#   bash scripts/uninstall-symlinks.sh <SOURCE_ROOT> <BUILD_ROOT> <prefix> <bindir> <datadir> <localedir> <mandir>
set -euo pipefail
SOURCE_ROOT="${1:?}"
BUILD_ROOT="${2:?}"
prefix="${3:?}"
bindir="${4:?}"
datadir="${5:?}"
localedir="${6:?}"
mandir="${7:?}"

for p in \
    "$bindir/ualias" \
    "$mandir/man1/ualias.1" \
    "$datadir/bash-completion/completions/ualias"
do
    if [ -L "$p" ]; then
        sudo rm -f "$p"
        printf "Removed %s\n" "$p"
    else
        printf "Skipped (not a symlink): %s\n" "$p"
    fi
done
