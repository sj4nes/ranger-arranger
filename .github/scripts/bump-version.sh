#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <patch|minor|major>" >&2
  exit 1
fi

BUMP="$1"
CARGO_TOML="Cargo.toml"
CHANGELOG="CHANGELOG.md"

CURRENT=$(grep -m 1 '^version = ' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/')
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

python3 - <<'PY' "$NEXT" "$CARGO_TOML"
import re, sys
version = sys.argv[1]
path = sys.argv[2]
with open(path, "r") as f:
    content = f.read()
content = re.sub(r'^version = ".*"', 'version = "' + version + '"', content, flags=re.MULTILINE)
with open(path, "w") as f:
    f.write(content)
PY

if [ -f "$CHANGELOG" ]; then
  TODAY=$(date +%Y-%m-%d)
  python3 - <<'PY' "$NEXT" "$TODAY" "$CHANGELOG"
import re, sys
version = sys.argv[1]
date = sys.argv[2]
path = sys.argv[3]
with open(path, "r") as f:
    content = f.read()
content = re.sub(r'^## \[Unreleased\]', '## [' + version + '] - ' + date, content, flags=re.MULTILINE)
content = "\n## [Unreleased]\n" + content
with open(path, "w") as f:
    f.write(content)
PY
fi

git add "$CARGO_TOML" "$CHANGELOG"
if [ -f Cargo.lock ]; then
  git add Cargo.lock
fi
git commit -m "chore: bump version to $NEXT"
git tag "v$NEXT"

# If PAT_TOKEN is provided, use it for the push
if [ -n "${PAT_TOKEN:-}" ] && [ -n "${GITHUB_REPOSITORY:-}" ]; then
  git remote set-url origin "https://x-access-token:${PAT_TOKEN}@github.com/${GITHUB_REPOSITORY}.git"
fi

if git remote get-url origin >/dev/null 2>&1; then
  git push origin main --tags
else
  echo "Skipping push: no origin remote configured"
fi
