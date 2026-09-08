# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Proptest coverage for `RANGE_AGG` aggregates across all four subtypes (`int8`, `int4`, `date`, `datetime`) in `tests/proptest_suite.rs`.
- Fuzz harness coverage for `RANGE_AGG` in `tests/fuzz_harness.rs`.
- Coverage slice tests for `func/predicates.rs` (`tests/coverage_slice_predicates.rs`, `tests/coverage_slice_predicates2.rs`): exercises `contains_point`, `contains_element`, `pred_binary`, and `pred_flag` across all subtypes including error paths.
- Coverage slice tests for `subtype/int4.rs`, `subtype/int8.rs`, `subtype/date.rs`, `subtype/datetime.rs` — all subtype modules now at 97–100% line coverage.
- Coverage slice tests for `func/construct.rs`, `func/extract.rs`, and `multirange_types.rs`.
- `coverage` task in `Justfile` for local `cargo llvm-cov` report generation.

### Changed
- `Justfile` `test` target now skips tests with "fuzz" in the name (`--skip fuzz`); `fuzz` runs as a separate step in `ci`.
- `Justfile` `ci` target runs `fmt-check clippy test fuzz` (fuzz after the filtered test step).
- CI coverage step uses `--lib --tests` instead of `--lib` alone, so integration test binaries are included in the coverage calculation.

### Fixed
- Clippy `unused-imports` warnings in new coverage test files (`coverage_slice_int4.rs`, `coverage_slice_int8.rs`, `coverage_slice_date.rs`, `coverage_slice_datetime.rs`, `coverage_slice_multirange_types.rs`).
- `normalize_components` made `pub` in `multirange_types.rs` for use by `range_agg` and tests.

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
