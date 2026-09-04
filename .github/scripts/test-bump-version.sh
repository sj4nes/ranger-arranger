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

# Shared fixture generators --------------------------------------------------
setUpBasic() {
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

  cat > "$WORK/manifest.json" <<'EOF'
{
  "name": "vsql_ranger_arranger",
  "version": "0.1.0"
}
EOF
}

setUpChangelog() {
  local body="$1"
  cat > "$WORK/CHANGELOG.md" <<EOF
# Changelog

${body}
EOF
}

cd "$WORK"

git init
git config user.email "test@test"
git config user.name "Test"

# ---- Basic bump tests (unchanged contract) ----

setUpBasic
setUpChangelog '## [Unreleased]

## [0.1.0]
- Initial release.'

echo "=== Test: patch bump ==="
"$BUMP_SCRIPT" patch

grep -q '^version = "0.1.1"' Cargo.toml || fail "Cargo.toml version not bumped to 0.1.1"
grep -q '"version": "0.1.1"' manifest.json || fail "manifest.json version not bumped to 0.1.1"
grep -q '^## \[0.1.1\]' CHANGELOG.md || fail "CHANGELOG.md missing [0.1.1] header"
grep -q '^## \[Unreleased\]' CHANGELOG.md || fail "CHANGELOG.md missing [Unreleased] after bump"
pass "patch bump"

setUpBasic
setUpChangelog '## [Unreleased]

## [0.1.0]
- Initial release.'

echo "=== Test: minor bump ==="
"$BUMP_SCRIPT" minor

grep -q '^version = "0.2.0"' Cargo.toml || fail "Cargo.toml version not bumped to 0.2.0"
grep -q '"version": "0.2.0"' manifest.json || fail "manifest.json version not bumped to 0.2.0"
grep -q '^## \[0.2.0\]' CHANGELOG.md || fail "CHANGELOG.md missing [0.2.0] header"
pass "minor bump"

setUpBasic
setUpChangelog '## [Unreleased]

## [0.1.0]
- Initial release.'

echo "=== Test: major bump ==="
"$BUMP_SCRIPT" major

grep -q '^version = "1.0.0"' Cargo.toml || fail "Cargo.toml version not bumped to 1.0.0"
grep -q '"version": "1.0.0"' manifest.json || fail "manifest.json version not bumped to 1.0.0"
grep -q '^## \[1.0.0\]' CHANGELOG.md || fail "CHANGELOG.md missing [1.0.0] header"
pass "major bump"

# ---- Hardening tests ----

setUpBasic
setUpChangelog '## [Unreleased]

### Added
- Multiline unreleased body.
- Second bullet.

## [0.1.0]
- Initial release.'

echo "=== Test: multiline unreleased preserved ==="
"$BUMP_SCRIPT" patch

grep -q '^version = "0.1.1"' Cargo.toml || fail "Cargo.toml not bumped to 0.1.1"
grep -q '^## \[0.1.1\] - ' CHANGELOG.md || fail "CHANGELOG.md missing [0.1.1] dated header"
grep -q '^### Added' CHANGELOG.md || fail "CHANGELOG.md multiline body lost"
grep -q 'Multiline unreleased body' CHANGELOG.md || fail "CHANGELOG.md unreleased bullet lost"
unreleased_count=$(grep -cFx '## [Unreleased]' CHANGELOG.md || true)
[ "$unreleased_count" -eq 1 ] || fail "expected 1 [Unreleased] header, got $unreleased_count"
pass "multiline unreleased preserved"

setUpBasic
setUpChangelog '## [0.1.0]
- Initial release.

## [Unreleased]

### Added
- Future work.'

echo "=== Test: rejected when [Unreleased] not first ==="
set +e
"$BUMP_SCRIPT" patch >/dev/null 2>&1
rc=$?
set -e
[ "$rc" -ne 0 ] || fail "bump should have failed when [Unreleased] is not first"
grep -q '^version = "0.1.0"' Cargo.toml || fail "Cargo.toml should not have been bumped on failure"
grep -q '^## \[Unreleased\]' CHANGELOG.md || fail "CHANGELOG.md should be unchanged on failure"
pass "reordered sections rejected"

setUpBasic
setUpChangelog '## [0.1.0]
- Initial release.'

echo "=== Test: rejected when no [Unreleased] ==="
set +e
"$BUMP_SCRIPT" patch >/dev/null 2>&1
rc=$?
set -e
[ "$rc" -ne 0 ] || fail "bump should have failed when [Unreleased] is missing"
grep -q '^version = "0.1.0"' Cargo.toml || fail "Cargo.toml should not have been bumped on failure"
pass "missing unreleased rejected"

setUpBasic
setUpChangelog '## [Unreleased]

## [Unreleased]

## [0.1.0]
- Initial release.'

echo "=== Test: rejected on duplicate [Unreleased] ==="
set +e
"$BUMP_SCRIPT" patch >/dev/null 2>&1
rc=$?
set -e
[ "$rc" -ne 0 ] || fail "bump should have failed on duplicate [Unreleased]"
grep -q '^version = "0.1.0"' Cargo.toml || fail "Cargo.toml should not have been bumped on failure"
pass "duplicate unreleased rejected"

setUpBasic
setUpChangelog '## [Unreleased]

## [0.1.0]
- Initial release.'

echo "=== Test: clean empty unreleased ==="
"$BUMP_SCRIPT" minor

grep -q '^version = "0.2.0"' Cargo.toml || fail "Cargo.toml not bumped to 0.2.0"
grep -q '^## \[0.2.0\] - ' CHANGELOG.md || fail "CHANGELOG.md missing [0.2.0] dated header"
unreleased_count=$(grep -cFx '## [Unreleased]' CHANGELOG.md || true)
[ "$unreleased_count" -eq 1 ] || fail "expected 1 [Unreleased] header after clean bump, got $unreleased_count"
unreleased_line=$(grep -nF '## [Unreleased]' CHANGELOG.md | head -1 | cut -d: -f1)
dated_line=$((unreleased_line + 1))
dated=$(sed -n "${dated_line}p" CHANGELOG.md)
grep -qF '## [0.2.0] - ' <(echo "$dated") || fail "dated section not immediately after empty unreleased"
pass "clean empty unreleased"

echo "=== All tests passed ==="
