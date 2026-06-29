# tools/release/create-tag.sh
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
# shellcheck source=tools/release/env.sh
. "$SCRIPT_DIR/env.sh"

usage() {
  cat <<EOF
Usage:
  tools/release/create-tag.sh <version> <release title without version>

Or:
  export VNUM=v4.1.2
  export VSTR="Okinawa Travel Book sample enrichment"
  tools/release/create-tag.sh

Example:
  tools/release/create-tag.sh v4.1.2 "Okinawa Travel Book sample enrichment"

Environment:
  VNUM / VSTR     Version and title when args are omitted
  SKIP_CHECK=1    Skip make check
EOF
}

if [ "$#" -lt 2 ] && { [ -z "${VNUM:-}" ] || [ -z "${VSTR:-}" ]; }; then
  usage
  exit 1
fi

resolve_release_args "$@"
NOTES_FILE="$(release_notes_file "$VERSION")"

require_cmd git
require_cmd cargo
require_cmd make

assert_on_main_branch
assert_cargo_version_matches "$VERSION"
assert_notes_file_exists "$NOTES_FILE"
assert_local_tag_absent "$VERSION"
assert_remote_tag_absent "$VERSION"
assert_worktree_has_changes

info "Release version : $VERSION"
info "Release title   : $TITLE"
info "Release notes   : $NOTES_FILE"
info "Repository      : $REPO"
info "Branch          : $MAIN_BRANCH"

if [ "${SKIP_CHECK:-0}" != "1" ]; then
  info "Running make check"
  make check
else
  info "Skipping make check because SKIP_CHECK=1"
fi

info "Staging changes"
git add .

if git diff --cached --quiet; then
  die "No staged changes after git add"
fi

COMMIT_MESSAGE="Release $VERSION — $TITLE"
TAG_MESSAGE="$VERSION $TITLE"

info "Creating commit: $COMMIT_MESSAGE"
git commit -m "$COMMIT_MESSAGE"

info "Creating annotated tag: $VERSION"
git tag -a "$VERSION" -m "$TAG_MESSAGE"

info "Pushing $MAIN_BRANCH"
git push "$REMOTE" "$MAIN_BRANCH"

info "Pushing tag $VERSION"
git push "$REMOTE" "$VERSION"

info "Done"
info "Next:"
info "  export VNUM=$VERSION VSTR=\"$TITLE\""
info "  tools/release/release.sh"