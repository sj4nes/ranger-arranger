#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <patch|minor|major>" >&2
  exit 1
fi

BUMP="$1"
CARGO_TOML="Cargo.toml"
CHANGELOG="CHANGELOG.md"
MANIFEST="manifest.json"

CURRENT=$(awk '
/^\[package\]/{in_pkg=1; next}
in_pkg && /^version = /{
  gsub(/.*version = "/, "")
  gsub(/".*/, "")
  print
  exit
}
/^\[/{in_pkg=0}
' "$CARGO_TOML")
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
# Only update the version under [package], not dependencies
lines = content.splitlines()
in_pkg = False
for i, line in enumerate(lines):
    if line.startswith("[package]"):
        in_pkg = True
    elif line.startswith("[") and not line.startswith("[package]"):
        in_pkg = False
    if in_pkg and line.startswith("version = "):
        lines[i] = f'version = "{version}"'
        break
content = "\n".join(lines) + "\n"
with open(path, "w") as f:
    f.write(content)
PY

# Update manifest.json version
python3 - <<'PY' "$NEXT" "$MANIFEST"
import json, sys
version = sys.argv[1]
path = sys.argv[2]
with open(path, "r") as f:
    data = json.load(f)
data["version"] = version
with open(path, "w") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
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

git add "$CARGO_TOML" "$CHANGELOG" "$MANIFEST"
if [ -f Cargo.lock ]; then
  git add Cargo.lock
fi
git commit -m "chore: bump version to $NEXT"
git tag "v$NEXT"

# Output version for workflow to capture
if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "version=$NEXT" >> "$GITHUB_OUTPUT"; fi
if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "tag=v$NEXT" >> "$GITHUB_OUTPUT"; fi
