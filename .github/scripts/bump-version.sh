#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <patch|minor|major>" >&2
  exit 1
fi

BUMP="$1"
CARGO_TOML="Cargo.toml"
CHANGELOG="CHANGELOG.md"

CURRENT=$(grep '^version = ' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/')
MAJOR=$(echo "$CURRENT" | cut -d. -f1)
MINOR=$(echo "$CURRENT" | cut -d. -f2)
PATCH=$(echo "$CURRENT" | cut -d. -f3)

case "$BUMP" in
  patch) PATCH=$((PATCH + 1)) ;;
  minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
  major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
  *) echo "Unknown bump: $BUMP" >&2; exit 1 ;;
esac

NEXT="$MAJOR.$MINOR.$PATCH"
echo "Bumping $CURRENT -> $NEXT"

perl -pi -e "s/^version\\s*=\\s*\".*\"/version = \\\"$NEXT\\\"/" "$CARGO_TOML"
cargo update -p vsql_ranger_arranger

if [ -f "$CHANGELOG" ]; then
  TODAY=$(date +%Y-%m-%d)
  perl -pi -e "s/^## \\[Unreleased\\]/## [$NEXT] - $TODAY/" "$CHANGELOG"
  printf '\n## [Unreleased]\n' | cat - "$CHANGELOG" > "$CHANGELOG.tmp" && mv "$CHANGELOG.tmp" "$CHANGELOG"
fi

git add "$CARGO_TOML" Cargo.lock "$CHANGELOG"
git commit -m "chore: bump version to $NEXT"
git tag "v$NEXT"
git push origin main --tags
