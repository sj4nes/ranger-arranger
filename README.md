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

The test suite is organized in four tiers, from unit to live server.

### Environment

Set `VILLAGESQL_BUILD_DIR` to the VillageSQL build tree before running any test or build command. The dev server, `mysql` client, and `cargo vsql` all resolve from there.

```bash
export VILLAGESQL_BUILD_DIR="$HOME/build/villagesql"
```

### Running the suite

```bash
cargo test --all-targets --all-features
```

This runs unit tests, ABI tests, and proptest suites against the local crate. For live-server integration:

```bash
# Start a dev server with the extension loaded
just dev-server

# Run the mysql-test harness against it
cd mysql-test
perl mysql-test-run.pl --suite=/absolute/path/to/vsql_ranger_arranger/mysql-test
```

Stop the dev server with `just dev-server-stop`.

### Regenerating expected output

When a test's behavior changes intentionally, regenerate the `.result` file from actual server output:

```bash
cd mysql-test
perl mysql-test-run.pl --suite=/absolute/path/to/vsql_ranger_arranger/mysql-test --record
```

Always review the diff before committing. The `.result` file is the contract — don't edit it by hand to match a broken implementation.

### Test files

| File | Coverage |
|------|----------|
| `t/ranger_arranger.test` | Round-trip persistence (INT8RANGE, DATERANGE), scheduling scenario with bookings table, anti-lossy decomposition, NULL-safe algebra, empty/infinity round-trip |

### Note on skill omission

This extension was built in ignorance of the `vsql-extension-builder` skill
and the test suite and CI gates were put together without reading it first.
The absence of skill-prescribed tracking artifacts (`.claude/tracking/`,
independent review pass, simplification agents) is a process gap worth
closing for future extensions, not a claim about this extension.

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

## Working with custom types

This extension defines four SQL custom types: `INT8RANGE`, `INT4RANGE`, `DATERANGE`, and `DATETIMERANGE`. These are not built-in SQL types — they are opaque values the extension understands how to construct, compare, decompose, and serialize.

### Reading values back

When you `SELECT` a range column or an expression returning a range type, you get the canonical string form:

```sql
SELECT DATERANGE_MAKE('2026-01-01', '2026-01-31', '[)');
-- Returns: [2026-01-01,2026-01-31)
```

The canonical form is lower-inclusive / upper-exclusive `[)` for discrete types (INT8RANGE, INT4RANGE, DATERANGE), and preserves the supplied bound inclusivity for continuous types (DATETIMERANGE). If you need a different display format, decompose with `<T>_LOWER` and `<T>_UPPER` rather than trying to CAST to a string.

### CAST to and from range types

The extension does not currently provide explicit CAST operations for converting between range types and other SQL types. You construct ranges from string literals or scalar arguments using the `<T>_MAKE` constructors, and you decompose them with the `_LOWER` / `_UPPER` accessors. If you need to load ranges from a text column or cast a range back to a display string, use the constructor and accessor functions directly — there is no implicit CAST path between a VARCHAR column and a range type.

### Storing in tables

Range types work as column types like any other VillageSQL custom type:

```sql
CREATE TABLE meetings (
    room VARCHAR(64),
    booked DATERANGE
);
```

Insert with the constructor, query with the predicates:

```sql
INSERT INTO meetings VALUES ('conf-a', DATERANGE_MAKE('2026-01-10','2026-01-12','[)'));
SELECT room FROM meetings WHERE DATERANGE_OVERLAPS(booked, DATERANGE_MAKE('2026-01-11','2026-01-13','[)'));
```

### Serialization transparency

The on-disk `.veb` format stores ranges as a header byte plus endpoint bytes plus bound-inclusivity flags, defined per subtype in `src/engine/flags.rs` and `src/engine/canonical.rs`. The serialized form is stable across installations of the same extension version — a `.veb` built from one checkout will load and produce identical range values on another server running the same extension version, provided the server version is compatible. Do not attempt to hand-edit `.veb` files; regenerate them from source.

## Security considerations

This extension parses user-supplied range literals and deserializes binary blob input from `.veb` files. The attack surface is:

- **Range literal parsing.** Every `<T>_MAKE` constructor and every predicate/extractor accepting a range argument goes through `src/engine/canonical.rs`, which validates the input against the subtype's expected format. Malformed input (wrong bracket, bad date string, out-of-range integer) is rejected with a typed error rather than silently producing a wrong value or crashing.
- **Serialized blob deserialization.** The `decode` path in `src/engine/canonical.rs` is symmetric with `encode`: encode(decode(bytes)) round-trips for every registered subtype, and malformed headers, truncated bodies, or mismatched endpoint counts fail closed with a typed error. The fuzz harness in `src/fuzz_api.rs` and `tests/fuzz_harness.rs` exercises this path with random byte buffers to ensure no input produces an undefined state.
- **Subtype boundary.** Each subtype validates its own endpoint domain: INT8RANGE rejects values outside i64, INT4RANGE rejects values outside i32, DATERANGE rejects out-of-calendar dates, and DATETIMERANGE validates its microsecond ordinal. There is no path where a value from one subtype is accepted by another's parser.
- **No server-global state.** The extension does not modify server-global variables, session state outside its own calls, or thread-local storage visible to other queries. Each VDF call is independent.

The dependency supply chain is audited in CI: `cargo audit --deny unmaintained` runs in the release workflow with documented exceptions for transitive dependencies through the vendored `vsql-rust-sdk` and the `mysql` crate. See `SECURITY.md` for the private reporting channel.

## Known limitations

- **No explicit CAST between range types and other SQL types.** You construct ranges with `<T>_MAKE` and decompose them with `<T>_LOWER` / `<T>_UPPER`. There is no `CAST(... AS INT8RANGE)` or automatic conversion from VARCHAR. This is a missing capability, not a design decision — the VEF surface for custom-type casts is limited in the current SDK.
- **Discrete types canonicalize to `[)`.** INT8RANGE, INT4RANGE, and DATERANGE always normalize to lower-inclusive / upper-exclusive form. If you construct `[1,5]`, it becomes `[1,6)` in storage and display. This is correct for the discrete subtype semantics, but users coming from PostgreSQL range types that preserve inclusive upper bounds should be aware the display form may differ from the input literal.
- **`sqlx` and other async MySQL clients are blocked.** The current VillageSQL server build does not support stable prepared statements, so `sqlx`'s prepare/execute path fails with `Prepared statement needs to be re-prepared` (error 1615). The supported Rust client path is the `mysql` crate (version 28) against a live server. See `docs/roadmap.md` for the planned resolution.
- **Pinned to a local `vsql-rust-sdk` checkout.** The extension depends on `buffer_size` and the `sql_query` preview, which are not yet in the published crates.io `0.0.1`. Builds require the vendored submodule at `vendor/vsql-rust-sdk`. A future published SDK release may change this.
- **No trigger-based exclusion constraints.** The extension provides range operators and accessors but does not enforce exclusion constraints through triggers. Concurrency-safe range exclusion is a server-side concern and is not in scope for this extension.
- **`.veb` files are version-bound.** A `.veb` built from one extension version will load on a server running the same extension version, but cross-version compatibility is not guaranteed. Regenerate `.veb` files from source when upgrading the extension.
