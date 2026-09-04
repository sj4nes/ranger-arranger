# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Hash functions for INT4RANGE (`int4_hash`), DATERANGE (`date_hash`), and DATETIMERANGE (`dt_hash`), wired into custom type registrations. Hash uses `DefaultHasher` over the encoded byte representation, matching the existing `int8_hash` pattern.

### Fixed
- Null input checks moved to VDF implementation entry points: every `_impl` function now calls `guard_null` at the top and returns `VdfReturn::null()` immediately if any argument is null, before delegating to its helper. This aligns with the requirement that null be rejected at the VDF impl boundary, not only inside helpers.
- Defense-in-depth guard in `bytes_to_range` (`src/engine/canonical.rs`): returns an empty range instead of underflowing when called with a buffer shorter than the header length. The only current caller already guards against empty input; this closes the gap for any future direct caller.

### Changed
- Added `.claude/tracking/cto_review.md`: independent review of the extension against the release checklist, with per-item PASS/FAIL verdicts and file:line evidence. Scratchpad only, not shipped.
- README: refined the "Note on skill omission" paragraph so shipped prose never mentions the reviewer role that is forbidden by the skill's vocabulary rules outside quoted user-facing strings.
- Source comments: replaced "release checklist" wording in `src/lib.rs` with neutral wording to align with the skill's vocabulary rules.

## [0.1.9] - 2026-09-04

## [0.1.8] - 2026-09-04

## [0.1.7] - 2026-09-04

## [0.1.6] - 2026-09-03

## [0.1.5] - 2026-09-03

## [0.1.4] - 2026-09-03

## [0.1.3] - 2026-09-03

## [0.1.2] - 2026-09-03

## [0.1.0]

### Added
- Initial public release candidate.
