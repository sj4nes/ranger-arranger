#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUMP_SCRIPT="$SCRIPT_DIR/bump-version.sh"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

pass() {
  echo "PASS: $*"
}

# Create a temp workspace
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

cat > "$WORK/Cargo.toml" <<'EOF'
[package]
name = "vsql_ranger_arranger"
version = "0.1.0"
edition = "2021"

[lib]

[dependencies.chrono]
version = "0.4"
EOF

mkdir -p "$WORK/src"
cat > "$WORK/src/lib.rs" <<'EOF'
pub fn dummy() {}
EOF

cat > "$WORK/CHANGELOG.md" <<'EOF'
# Changelog

## [Unreleased]

## [0.1.0]
- Initial release.
EOF

cd "$WORK"

git init
git config user.email "test@test"
git config user.name "Test"

# Test patch bump
echo "=== Test: patch bump ==="
"$BUMP_SCRIPT" patch

grep -q '^version = "0.1.1"' Cargo.toml || fail "Cargo.toml version not bumped to 0.1.1"
grep -q '^## \[0.1.1\]' CHANGELOG.md || fail "CHANGELOG.md missing [0.1.1] header"
grep -q '^## \[Unreleased\]' CHANGELOG.md || fail "CHANGELOG.md missing [Unreleased] after bump"
pass "patch bump"

# Reset for minor bump test
cat > "$WORK/Cargo.toml" <<'EOF'
[package]
name = "vsql_ranger_arranger"
version = "0.1.0"
edition = "2021"

[lib]

[dependencies.chrono]
version = "0.4"
EOF

mkdir -p "$WORK/src"
cat > "$WORK/src/lib.rs" <<'EOF'
pub fn dummy() {}
EOF

cat > "$WORK/CHANGELOG.md" <<'EOF'
# Changelog

## [Unreleased]

## [0.1.0]
- Initial release.
EOF

echo "=== Test: minor bump ==="
"$BUMP_SCRIPT" minor

grep -q '^version = "0.2.0"' Cargo.toml || fail "Cargo.toml version not bumped to 0.2.0"
grep -q '^## \[0.2.0\]' CHANGELOG.md || fail "CHANGELOG.md missing [0.2.0] header"
pass "minor bump"

# Reset for major bump test
cat > "$WORK/Cargo.toml" <<'EOF'
[package]
name = "vsql_ranger_arranger"
version = "0.1.0"
edition = "2021"
EOF

cat > "$WORK/CHANGELOG.md" <<'EOF'
# Changelog

## [Unreleased]

## [0.1.0]
- Initial release.
EOF

echo "=== Test: major bump ==="
"$BUMP_SCRIPT" major

grep -q '^version = "1.0.0"' Cargo.toml || fail "Cargo.toml version not bumped to 1.0.0"
grep -q '^## \[1.0.0\]' CHANGELOG.md || fail "CHANGELOG.md missing [1.0.0] header"
pass "major bump"

echo "=== All tests passed ==="
