#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Sync gettext catalogs via zfr translate --sync.
# Invoked as: bash scripts/posync.sh <SOURCE_ROOT> <BUILD_ROOT>
set -euo pipefail
SOURCE_ROOT="${1:-${MESON_SOURCE_ROOT:-.}}"
cd "$SOURCE_ROOT"
exec zfr translate --sync
