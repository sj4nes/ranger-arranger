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
  python3 - "$NEXT" "$TODAY" "$CHANGELOG" <<'PY'
import sys

version = sys.argv[1]
date = sys.argv[2]
path = sys.argv[3]

with open(path, "r") as f:
    content = f.read()

lines = content.splitlines(keepends=True)

# --- Find and validate ## [Unreleased] ---
unreleased_idx = None
for i, line in enumerate(lines):
    if line.strip() == "## [Unreleased]":
        if unreleased_idx is not None:
            print(
                f"Error: multiple '## [Unreleased]' headers in {path}",
                file=sys.stderr,
            )
            sys.exit(1)
        unreleased_idx = i

if unreleased_idx is None:
    print(
        f"Error: no '## [Unreleased]' header found in {path}",
        file=sys.stderr,
    )
    sys.exit(1)

# Must be the first section header (skip title lines and blanks)
for i in range(unreleased_idx):
    if lines[i].strip().startswith("## ["):
        print(
            f"Error: '## [Unreleased]' is not the first section in {path} "
            f"(found '{lines[i].strip()}' before it)",
            file=sys.stderr,
        )
        sys.exit(1)

# --- Find section boundaries ---
section_end = len(lines)
for i in range(unreleased_idx + 1, len(lines)):
    if lines[i].strip().startswith("## ["):
        section_end = i
        break

preamble = lines[:unreleased_idx]
unreleased_body = lines[unreleased_idx + 1 : section_end]
rest = lines[section_end:]

# --- Build output ---
new_lines = list(preamble)
new_lines.append("## [Unreleased]\n")
new_lines.append("\n")
new_lines.append(f"## [{version}] - {date}\n")
new_lines.append("\n")

# Skip leading blank lines from the old unreleased body
body_start = 0
while body_start < len(unreleased_body) and unreleased_body[body_start].strip() == "":
    body_start += 1
new_lines.extend(unreleased_body[body_start:])
new_lines.extend(rest)

result = "".join(new_lines)
if not result.endswith("\n"):
    result += "\n"

with open(path, "w") as f:
    f.write(result)
PY
fi

git add "$CARGO_TOML" "$CHANGELOG" "$MANIFEST"
if [ -f Cargo.lock ]; then
  git add Cargo.lock
fi
git commit -m "chore: bump version to $NEXT"
# Replace any existing tag at this version so re-runs and retries are idempotent.
if git rev-parse --verify "v$NEXT" >/dev/null 2>&1; then
  git tag -d "v$NEXT" >/dev/null 2>&1
fi
git tag "v$NEXT"

# Output version for workflow to capture
if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "version=$NEXT" >> "$GITHUB_OUTPUT"; fi
if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "tag=v$NEXT" >> "$GITHUB_OUTPUT"; fi
