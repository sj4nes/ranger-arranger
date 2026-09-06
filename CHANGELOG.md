# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-09-06

### Added
- Four multirange types: `INT8MULTIRANGE`, `INT4MULTIRANGE`, `DATEMULTIRANGE`, `DATETIMEMULTIRANGE`.
- Multirange VDFs: `<MR>_MAKE`, `<MR>_LOWER`, `<MR>_OVERLAPS`, `<MR>_CONTAINS_RANGE`, `<MR>_INTERSECT`, `<MR>_MERGE`, `<MR>_DIFFERENCE`.
- `examples/booking_multirange_demo.rs`: DATEMULTIRANGE booking scenario exercising `MAKE`, `OVERLAPS`, `MERGE`, `DIFFERENCE`, and `INTERSECT`.
- Expanded `mysql-test/t/multirange.test` with multirange algebra coverage and recorded expected output.
- Coverage gap tests in `tests/multirange_gaps.rs` for null propagation, argument-count errors, reversed bounds, empty decode, and subtype-specific round-trips.
- CI coverage baseline using `cargo llvm-cov` with an `ignore-filename-regex` for vendored SDK code and a `fail-under-lines` floor.

### Fixed
- Null input checks moved to VDF implementation entry points: every `_impl` function now calls `guard_null` at the top and returns `VdfReturn::null()` immediately if any argument is null, before delegating to its helper. This aligns with the requirement that null be rejected at the VDF impl boundary, not only inside helpers.
- Defense-in-depth guard in `bytes_to_range` (`src/engine/canonical.rs`): returns an empty range instead of underflowing when called with a buffer shorter than the header length. The only current caller already guards against empty input; this closes the gap for any future direct caller.

### Changed
- README documents the new multirange types, their function surface, and the new booking demo.

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
