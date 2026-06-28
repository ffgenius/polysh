#!/usr/bin/env bash
set -euo pipefail

# ── Helpers ──────────────────────────────────────────────────────────────

RED='\033[0;31m'
NC='\033[0m' # No Color

die() {
  echo -e "${RED}error:${NC} $*" >&2
  exit 1
}

bump_patch() { echo "$1" | awk -F. '{print $1"."$2"."$3+1}'; }
bump_minor() { echo "$1" | awk -F. '{print $1"."$2+1".0"}'; }
bump_major() { echo "$1" | awk -F. '{print $1+1".0.0"}'; }

# ── Read current version ─────────────────────────────────────────────────

CARGO_TOML="Cargo.toml"
CURRENT_VERSION=$(grep -oP '^version\s*=\s*"\K[^"]+' "$CARGO_TOML")
if [ -z "$CURRENT_VERSION" ]; then
  die "could not parse version from $CARGO_TOML"
fi

echo "current version: $CURRENT_VERSION"
echo ""
echo "select bump type:"
echo "  1) patch  →  $(bump_patch "$CURRENT_VERSION")"
echo "  2) minor  →  $(bump_minor "$CURRENT_VERSION")"
echo "  3) major  →  $(bump_major "$CURRENT_VERSION")"
echo "  4) custom"
echo ""
read -rp "choice [1-4] (default: 1): " CHOICE
CHOICE="${CHOICE:-1}"

case "$CHOICE" in
  1) NEW_VERSION=$(bump_patch "$CURRENT_VERSION") ;;
  2) NEW_VERSION=$(bump_minor "$CURRENT_VERSION") ;;
  3) NEW_VERSION=$(bump_major "$CURRENT_VERSION") ;;
  4)
    read -rp "enter new version: " NEW_VERSION
    if [ -z "$NEW_VERSION" ]; then
      die "version cannot be empty"
    fi
    ;;
  *) die "invalid choice: $CHOICE" ;;
esac

echo ""
echo "new version:  $CURRENT_VERSION  →  $NEW_VERSION"
read -rp "proceed? [Y/n] " CONFIRM
case "${CONFIRM:-y}" in
  [Yy]*) ;;
  *) echo "aborted"; exit 0 ;;
esac

# ── Update version in Cargo.toml ────────────────────────────────────────

echo ""
echo "updating Cargo.toml ..."
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"

# ── Commit and tag ───────────────────────────────────────────────────────

echo ""
echo "committing and tagging ..."

git add Cargo.toml
git commit -m "release: v${NEW_VERSION}"
git tag -a "v${NEW_VERSION}" -m "v${NEW_VERSION}"

echo ""
echo "done! Run the following to trigger CI:"
echo "  git push --follow-tags"
