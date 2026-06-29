#!/usr/bin/env bash
# tools/release/full-release.sh
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
# shellcheck source=tools/release/env.sh
. "$SCRIPT_DIR/env.sh"

usage() {
  cat <<EOF
Usage:
  tools/release/full-release.sh <version> <release title without version>

Or:
  export VNUM=v4.1.2
  export VSTR="Okinawa Travel Book sample enrichment"
  tools/release/full-release.sh

Runs create-tag.sh then release.sh with the same version and title.

Environment:
  VNUM / VSTR     Version and title when args are omitted
  SKIP_CHECK=1    Skip make check (passed through to create-tag.sh)
  WAIT_RELEASE_SECONDS / WAIT_ASSETS_SECONDS   Passed through to release.sh
EOF
}

if [ "$#" -ge 2 ]; then
  export VNUM="$(with_v "$1")"
  shift
  export VSTR="$*"
elif [ -n "${VNUM:-}" ] && [ -n "${VSTR:-}" ]; then
  export VNUM="$(with_v "$VNUM")"
  export VSTR
else
  usage
  exit 1
fi

info "Full release: $VNUM — $VSTR"

"$SCRIPT_DIR/create-tag.sh"
"$SCRIPT_DIR/release.sh"

info "Full release complete"
