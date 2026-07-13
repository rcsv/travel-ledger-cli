#!/usr/bin/env bash
# tools/desktop/dev.sh
# Start Travel Ledger Desktop (Tauri) in development mode for manual smoke checks.
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT_DIR="$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)"
DESKTOP_DIR="$ROOT_DIR/desktop"

usage() {
  cat <<EOF
Usage:
  tools/desktop/dev.sh

Installs desktop npm dependencies and runs the Tauri development app:

  cd desktop
  npm install
  npm run tauri dev

Environment:
  SKIP_INSTALL=1   Skip npm install
EOF
}

die() {
  echo "ERROR: $*" >&2
  exit 1
}

info() {
  echo "==> $*"
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  usage
  exit 0
fi

if [ "$#" -ne 0 ]; then
  usage
  exit 1
fi

[ -d "$DESKTOP_DIR" ] || die "Desktop directory not found: $DESKTOP_DIR"
[ -f "$DESKTOP_DIR/package.json" ] || die "package.json not found: $DESKTOP_DIR/package.json"

require_cmd npm
require_cmd cargo
require_cmd rustc

cd "$DESKTOP_DIR"

if [ "${SKIP_INSTALL:-0}" = "1" ]; then
  info "Skipping npm install (SKIP_INSTALL=1)"
else
  info "Installing desktop npm dependencies"
  npm install
fi

info "Starting Tauri development app"
info "Stop with Ctrl-C"
exec npm run tauri dev
