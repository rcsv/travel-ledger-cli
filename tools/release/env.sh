# tools/release/env.sh
#!/usr/bin/env bash
# Common helpers for Caglla.Travel CLI release scripts.
# This file is meant to be sourced from other scripts.

set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"

REPO="${REPO:-rcsv/travel-ledger-cli}"
REMOTE="${REMOTE:-origin}"
MAIN_BRANCH="${MAIN_BRANCH:-master}"
PACKAGE_NAME="${PACKAGE_NAME:-travel-ledger-cli}"

cd "$ROOT_DIR"

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

strip_v() {
  printf '%s' "${1#v}"
}

with_v() {
  case "$1" in
    v*) printf '%s' "$1" ;;
    *)  printf 'v%s' "$1" ;;
  esac
}

cargo_version() {
  awk -F '"' '/^version = / { print $2; exit }' Cargo.toml
}

release_notes_file() {
  version_with_v="$(with_v "$1")"
  printf 'docs/releases/%s-notes.md' "$version_with_v"
}

assert_on_main_branch() {
  branch="$(git rev-parse --abbrev-ref HEAD)"
  [ "$branch" = "$MAIN_BRANCH" ] || die "Current branch is '$branch', expected '$MAIN_BRANCH'"
}

assert_cargo_version_matches() {
  version="$(strip_v "$1")"
  actual="$(cargo_version)"
  [ "$actual" = "$version" ] || die "Cargo version is '$actual', expected '$version'"
}

assert_notes_file_exists() {
  notes_file="$1"
  [ -f "$notes_file" ] || die "Release notes not found: $notes_file"
}

assert_local_tag_absent() {
  version="$(with_v "$1")"
  if git rev-parse "$version" >/dev/null 2>&1; then
    die "Local tag already exists: $version"
  fi
}

assert_remote_tag_absent() {
  version="$(with_v "$1")"
  if git ls-remote --exit-code --tags "$REMOTE" "refs/tags/$version" >/dev/null 2>&1; then
    die "Remote tag already exists: $version"
  fi
}

assert_worktree_has_changes() {
  if [ -z "$(git status --porcelain)" ]; then
    die "No changes to commit"
  fi
}

expected_asset_names() {
  version_without_v="$(strip_v "$1")"

  cat <<EOF
${PACKAGE_NAME}-${version_without_v}-linux-amd64.tar.gz
${PACKAGE_NAME}-${version_without_v}-macos-arm64.tar.gz
${PACKAGE_NAME}-${version_without_v}-windows-amd64.zip
EOF
}

# Sets VERSION and TITLE from positional args or VNUM/VSTR environment variables.
resolve_release_args() {
  if [ "$#" -ge 2 ]; then
    VERSION="$(with_v "$1")"
    shift
    TITLE="$*"
  else
    [ -n "${VNUM:-}" ] || die "Version is required: pass arg or set VNUM"
    [ -n "${VSTR:-}" ] || die "Release title is required: pass arg or set VSTR"

    VERSION="$(with_v "$VNUM")"
    TITLE="$VSTR"
  fi
}
