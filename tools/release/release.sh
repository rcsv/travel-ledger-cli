# tools/release/release.sh
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
# shellcheck source=tools/release/env.sh
. "$SCRIPT_DIR/env.sh"

usage() {
  cat <<EOF
Usage:
  tools/release/release.sh <version> <release title without version>

Or:
  export VNUM=v4.1.2
  export VSTR="Okinawa Travel Book sample enrichment"
  tools/release/release.sh

Example:
  tools/release/release.sh v4.1.2 "Okinawa Travel Book sample enrichment"

Environment:
  VNUM / VSTR                Version and title when args are omitted
  WAIT_RELEASE_SECONDS=180   Seconds to wait for auto-created GitHub Release
  WAIT_ASSETS_SECONDS=300    Seconds to wait for 3OS assets
EOF
}

if [ "$#" -lt 2 ] && { [ -z "${VNUM:-}" ] || [ -z "${VSTR:-}" ]; }; then
  usage
  exit 1
fi

resolve_release_args "$@"
RELEASE_TITLE="$VERSION $TITLE"
NOTES_FILE="$(release_notes_file "$VERSION")"

WAIT_RELEASE_SECONDS="${WAIT_RELEASE_SECONDS:-180}"
WAIT_ASSETS_SECONDS="${WAIT_ASSETS_SECONDS:-300}"

require_cmd gh
require_cmd git

assert_notes_file_exists "$NOTES_FILE"

info "Release version : $VERSION"
info "Release title   : $RELEASE_TITLE"
info "Release notes   : $NOTES_FILE"
info "Repository      : $REPO"

info "Waiting for GitHub Release to appear, if workflow creates it automatically"

elapsed=0
release_exists=0

while [ "$elapsed" -le "$WAIT_RELEASE_SECONDS" ]; do
  if gh release view "$VERSION" --repo "$REPO" >/dev/null 2>&1; then
    release_exists=1
    break
  fi

  sleep 10
  elapsed=$((elapsed + 10))
done

if [ "$release_exists" -eq 1 ]; then
  info "GitHub Release exists; waiting for assets before editing title/notes"
else
  info "GitHub Release not found yet; waiting for workflow to create it and upload assets"
fi

info "Waiting for expected assets"

elapsed=0
missing=""

while [ "$elapsed" -le "$WAIT_ASSETS_SECONDS" ]; do
  if ! gh release view "$VERSION" --repo "$REPO" >/dev/null 2>&1; then
    info "Release not visible yet; waiting for workflow"
    sleep 15
    elapsed=$((elapsed + 15))
    continue
  fi

  release_exists=1
  missing=""

  for asset in $(expected_asset_names "$VERSION"); do
    if ! gh release view "$VERSION" --repo "$REPO" --json assets --jq '.assets[].name' | grep -Fx "$asset" >/dev/null 2>&1; then
      missing="$missing $asset"
    fi
  done

  if [ -z "$missing" ]; then
    info "All expected assets are uploaded"
    break
  fi

  info "Still waiting for assets:$missing"
  sleep 15
  elapsed=$((elapsed + 15))
done

if [ -n "${missing:-}" ]; then
  die "Some expected assets are still missing:$missing"
fi

if [ "$release_exists" -eq 1 ]; then
  info "Editing GitHub Release title and notes (after assets are ready)"
  gh release edit "$VERSION" \
    --repo "$REPO" \
    --title "$RELEASE_TITLE" \
    --notes-file "$NOTES_FILE"
else
  info "GitHub Release was not auto-created. Creating it manually."
  gh release create "$VERSION" \
    --repo "$REPO" \
    --verify-tag \
    --title "$RELEASE_TITLE" \
    --notes-file "$NOTES_FILE"
fi

info "Final release view"
gh release view "$VERSION" \
  --repo "$REPO" \
  --json tagName,name,assets,url

info "Done"