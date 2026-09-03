# ranger-arranger — Range types for VillageSQL/VEF

Reference implementation of range-type extensions for VillageSQL/VEF,
built against the `vsql-rust-sdk`.

## Types

| SQL type        | subtype     | discrete | endpoint width |
|-----------------|-------------|----------|----------------|
| `INT8RANGE`     | `Int8Ops`   | yes      | 8 |
| `INT4RANGE`     | `Int4Ops`   | yes      | 4 |
| `DATERANGE`     | `DateOps`   | yes      | 8 (days) |
| `DATETIMERANGE` | `DateTimeOps`| no      | 8 (micros) |

Discrete types canonicalize to lower-inclusive / upper-exclusive `[)`;
continuous (`DATETIMERANGE`) preserve the supplied bound inclusivity.

## Functions

All functions are `deterministic: true` and NULL-explicit.

VEF keys VDFs by `(name, argument types)`, so each subtype exposes its
own PostgreSQL-style typed names (`int8range()` / `daterange()`). Every
subtype exposes the same surface:

- Constructors: `<T>_MAKE`, `<T>_EMPTY`.
- Predicates: `<T>_OVERLAPS`, `<T>_CONTAINS_RANGE`, `<T>_ADJACENT`, `<T>_BEFORE`,
  `<T>_EQUALS`, `<T>_ISEMPTY`.
- Extract: `<T>_LOWER`, `<T>_UPPER`, `<T>_LOWER_INC`, `<T>_UPPER_INC`.
- Set ops: `<T>_INTERSECT`, `<T>_MERGE`, `<T>_UNION`, `<T>_DIFFERENCE`
  (anti-lossy: returns a JSON array of pieces, never a single lossy range),
  `<T>_LENGTH`.

## Usage

The examples below use real SQL from `mysql-test/t/ranger_arranger.test`.

### 1. Scheduling: find free slots from booked intervals

```sql
-- Booked meetings for one room on 2026-01-01
SELECT DATERANGE_DIFFERENCE(
    DATERANGE_MAKE('2026-01-01','2026-01-31','[)'),
    DATERANGE_MAKE('2026-01-10','2026-01-20','[)')
);
-- Returns: [[2026-01-01,2026-01-10),[2026-01-20,2026-01-31)]

-- Check whether a proposed meeting overlaps an existing booking
SELECT DATERANGE_INTERSECT(
    DATERANGE_MAKE('2026-01-15','2026-01-20','[)'),
    DATERANGE_MAKE('2026-01-10','2026-01-12','[)')
);
-- Returns: empty (no conflict)
```

This is the most common operational pattern: a table of booked ranges, and a query that derives the complement. Without range types, this requires tedious endpoint arithmetic or recursive CTEs.

### 2. Anti-lossy gap decomposition

```sql
-- Subtract one integer range from another
SELECT INT8RANGE_DIFFERENCE(
    INT8RANGE_MAKE(1,10,'[)'),
    INT8RANGE_MAKE(3,6,'[)')
);
-- Returns: [[1,3),[6,10)]

-- A single subtraction can produce 0, 1, or 2 pieces.
-- The extension never collapses them into a lossy single range.
```

This is what makes the type useful for accounting, version validity, and feature-flag windows: the missing piece is always exact.

### 3. NULL-safe algebra

```sql
-- Any NULL input propagates through predicates and constructors.
SELECT INT8RANGE_OVERLAPS(NULL, INT8RANGE_MAKE(1,5,'[)'));
-- Returns: NULL

SELECT INT8RANGE_MAKE(NULL, 5, '[]');
-- Returns: NULL
```

All VDFs are NULL-explicit, so the extension behaves like ordinary SQL functions rather than crashing or inventing values.

## Why range types?

The usual SQL pattern for intervals is two columns — `start` and `end` — with a handshake agreement that the application layer enforces. That is not a data model; it is a maintenance burden. Every overlap check, gap calculation, or containment test becomes endpoint arithmetic, self-joins, or recursive CTEs that are easy to get wrong at the edges.

Range types treat an interval as one scalar value. You construct it with `RANGE_MAKE`, test it with `OVERLAPS` and `CONTAINS_RANGE`, and compute gaps with `DIFFERENCE` instead of reimplementing set theory in application code.

For discrete types, the canonical lower-inclusive / upper-exclusive `[)` form is not an implementation detail — it is the mathematically correct representation. `[1,5]` normalizes to `[1,6)` so equality and hashing are stable regardless of how the range was written. You do not have to remember the original literal and hope comparisons behave; the engine guarantees it.

Use this when you need to find free slots in a schedule, enforce exclusion through generated columns, or query temporal data without losing information to lossy set operations. If you are still splitting ranges into two columns and writing your own overlap logic, you are not using SQL — you are fighting it.

This extension is designed for VillageSQL/VEF, so it runs as a server-side VDF rather than a client-side helper.

## Design choices

- Canonical storage: discrete types are stored as lower-inclusive / upper-exclusive `[)`, so equality and hashing are stable regardless of how the range was constructed.
- Anti-lossy set ops: `DIFFERENCE` returns a JSON array of ranges, never a single lossy summary.
- NULL-explicit: every function follows SQL's three-valued logic. `NULL` in, `NULL` out.
- One subtype, one registration: each range family has distinct function names, so the server never collides on `(name, argument types)`.

## Build & load

```bash
cargo build                       # target/debug/libvsql_ranger_arranger.dylib
cargo vsql install                # package .veb -> $VillageSQL_BUILD_DIR/veb_output_directory
# then, in the server:
INSTALL EXTENSION vsql_ranger_arranger;
```

Verify it loaded:

```sql
SELECT * FROM INFORMATION_SCHEMA.EXTENSIONS WHERE EXTENSION_NAME = 'vsql_ranger_arranger'\g
```

**Known client quirk:** with `mysql` client `8.4.11-villagesql-0.0.7-dev-a5d49e67f99` on macOS arm64, interactive lowercase `select` followed by a `)` inside a string literal (e.g. `'[)'`) can trigger a client-side parse error (`near 'selec'`) even though batch mode and the server accept it fine. Workaround: use uppercase `SELECT`, or append `\g`, or pipe the statement from a file/`-e`.

Pinned to the local `vsql-rust-sdk` checkout (ahead of published crates.io `0.0.1`:
has `buffer_size` + the `sql_query` preview). `cargo vsql` is the SDK's packaging tool;
the `.veb` it produces is what the server loads.

## Testing

Four independent tiers, from unit to live server:

- Unit tests: `cargo test` covers engine algebra, canonicalization, and subtype behavior.
- ABI tests: drives the exact `func!` wrapper builders with real `InValue`/`VdfReturn` values, decoding returned custom-type bytes back to literals. Pins the VEF contract without a live server.
- Property tests: proptest + a brute-force `BTreeSet` oracle checks intersect / merge / difference / contains / overlaps / adjacent across thousands of random ranges.
- Live server: `mysql-test/r/ranger_arranger.result` captures real server output for all four subtypes.

## Demo: conference room booking (Rust + `mysql` crate)

A working example is in `examples/booking_demo.rs`. It uses the `mysql` crate
(version `28`) against a running VillageSQL server and demonstrates:

- conflict detection with `DATERANGE_OVERLAPS`
- free-slot discovery by walking stored bookings ordered by lower bound
- fit verification with `NOT EXISTS` + `DATERANGE_OVERLAPS`

Run it:

```bash
DATABASE_URL='mysql://root@127.0.0.1:3307/test_ranger' cargo run --example booking_demo
```

This is the current supported path for Rust client code. `sqlx` is tracked in
`docs/roadmap.md` as a future improvement once the server exposes stable
prepared-statement support.

## Fuzzing

Two layers share one harness (`src/fuzz_api.rs`):

1. Stable offline fuzz: `cargo test --test fuzz_harness` runs ~150k random byte buffers through encode/decode/deserialize and differential algebra checks.
2. libFuzzer: `cargo +nightly fuzz run bytes` and `cargo +nightly fuzz run algebra` for raw-byte and random-range targets.

Fuzzing found and fixed 6 real bugs before release: empty-header form mismatch, version byte dropped, infinite-bound ordinal corruption, empty-range byte instability, sign-extension in ordinal read, and a merge-oracle mismatch.

## Benchmarking

`benches/engine_bench.rs` (criterion) profiles hot paths: `encode`, `decode`, `to_range`, in-memory algebra, and set-op serialization. These benchmarks are used to guard against regressions, not to promise specific speedups.

## Honesty boundary

This extension does NOT provide trigger-based exclusion constraints, and it does NOT enforce concurrent-write integrity. `RANGE_ASSERT_AVAILABLE` availability windows are deferred. The full list of guarantees and where they stop is in `docs/epic6-integrity-honesty.md`; the indexing work-around for fast overlap queries is in `docs/epic5-indexing-cookbook.md`.
