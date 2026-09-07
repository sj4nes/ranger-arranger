# Multirange completion plan

Goal: make multiranges the one-scalar primitive for schedules, bookings,
exclusion constraints, and version windows — without forcing CTEs or
application-side loop unrolling.

Current surface: `MAKE`, `LOWER`, `OVERLAPS`, `CONTAINS_RANGE`, `INTERSECT`,
`MERGE`, `DIFFERENCE` across four types.

Target surface: add element containment, `UNION`, `UPPER`/`UPPER_INC`/`BOUNDS`/`ISEMPTY`/`LENGTH`, `range_agg`/unnest, and an exclusion-constraint primitive.

## Slice 1 — element containment + multirange UNION

### 1a. `range <@ multirange` / `multirange @> range`

SQL shape:
- `INT8RANGE <@ INT8MULTIRANGE` → INT (0/1)
- `INT4RANGE <@ INT4MULTIRANGE` → INT
- `DATERANGE <@ DATEMULTIRANGE` → INT
- `DATETIMERANGE <@ DATETIMEMULTIRANGE` → INT

Engine work:
- `src/engine/mod.rs` — new `element_contained_by(range, multirange_bytes) -> bool`
  - decode multirange via `mr_decode_to_vec`
  - binary search or linear scan for a component that fully contains the range
- `src/multirange_types.rs` — expose `int8mr_contains_element`, `int4mr_contains_element`, `date_mr_contains_element`, `dtmr_contains_element` as typed wrappers
- `src/func/predicates.rs` — new `multirange_contains_element<T>` builder; mirror `pred_binary` pattern but second arg is multirange custom, first is range custom
- `src/lib.rs` — register 8 new VDFs: `<MR>_CONTAINS_ELEMENT` and `<MR>_IS_CONTAINED_BY` for each type

Tests:
- unit test in `tests/multirange_gaps.rs`: single range against single-component multirange
- unit test: range against empty multirange -> false
- unit test: range against multi-component multirange where middle component contains it
- proptest in `tests/proptest_suite.rs`: round-trip `contains_element` vs brute-force decode+overlaps

### 1b. `<MR>_UNION`

SQL shape:
- `INT8MULTIRANGE_UNION(a, b)` → binary multirange
- Same for INT4, DATE, DATETIME

Engine work:
- `src/engine/mod.rs` — new `multirange_union(a, b) -> Vec<u8>`
  - decode both via `mr_decode_to_vec`
  - concatenate, normalize, encode
- `src/multirange_types.rs` — typed `*_union` wrappers
- `src/func/setops.rs` — new `multirange_binary_set` wrapper `multirange_union`; existing wrapper already supports arbitrary set ops

Tests:
- unit: union of disjoint multiranges -> normalized combined
- unit: union of overlapping multiranges -> merged, deduplicated
- unit: union with empty -> input preserved
- proptest: union is inverse of difference for random multiranges

## Slice 2 — multirange accessor/predicate completion

### 2a. `<MR>_UPPER`, `<MR>_UPPER_INC`, `<MR>_BOUNDS`

Engine work:
- `src/func/extract.rs` — new `multirange_upper`, `multirange_upper_inc`, `multirange_bounds`
  - `UPPER`/`UPPER_INC`: decode via `mr_decode_to_vec`, take last component
  - `BOUNDS`: return `[lower, upper)` style text from last component
- `src/lib.rs` — register for each type

Tests:
- unit: upper of `{[1,5),[10,15)}` -> `15`
- unit: upper_inc of `{[1,5]}` -> `1` (canonicalized)
- unit: bounds text round-trips back to same multirange literal

### 2b. `<MR>_ISEMPTY`, `<MR>_LENGTH`

Engine work:
- `src/func/predicates.rs` — new `multirange_isempty`
- `src/func/setops.rs` — new `multirange_length_for`
- `src/lib.rs` — register

Tests:
- unit: `empty` is empty; `{[1,5)}` is not
- unit: length of empty -> 0; length of `{[1,5)}` -> 4 for int8

## Slice 3 — multirange accessors: `NTH`, `LENGTH`, `RANGE_AGG`

### 3a. `<MR>_NTH(mr, n)`

SQL shape:
- `INT8MULTIRANGE_NTH(m, n)` -> binary multirange containing exactly one component
- Same for INT4, DATE, DATETIME
- `n` is 1-based; out-of-range/empty -> NULL

Engine work:
- `src/func/setops.rs` — new `multirange_nth` helper: decode -> pick component -> re-encode single range
- `src/lib.rs` — register for each type

Tests:
- unit: nth of `{[1,3),[7,10)}` -> `{[7,10)}`
- unit: nth out of range -> NULL
- unit: nth on empty multirange -> NULL

### 3b. `<MR>_LENGTH`

SQL shape:
- `INT8MULTIRANGE_LENGTH(m)` -> INT component count

Engine work:
- `src/func/setops.rs` — new `multirange_length` helper: decode -> count components
- `src/lib.rs` — register for each type

Tests:
- unit: length of `{}` -> 0
- unit: length of `{[1,3),[7,10)}` -> 2

### 3c. `<MR>_RANGE_AGG`

SQL shape:
- `INT8MULTIRANGE_RANGE_AGG(range, ...)` -> multirange aggregating all input ranges
- Accepts varargs multirange values; decodes each, concatenates components, normalizes, re-encodes

Engine work:
- `src/func/setops.rs` — new `multirange_range_agg` helper
- `src/lib.rs` — register for each type

Tests:
- unit: aggregate disjoint ranges -> merged multirange
- unit: aggregate overlapping ranges -> deduped multirange
- unit: aggregate NULL -> ignores NULLs

## Slice 4 — exclusion constraint primitive

### 4a. `MR_EXCLUDING`

SQL shape:
- `INT8MULTIRANGE_EXCLUDING(schedule, booking)` -> schedule minus booking

This is `DIFFERENCE` by another name for multiranges, but named to match the
constraint-story vocabulary. Defer unless there is an actual `EXCLUDE USING`
gist/index path exposed by the server.

## File plan

```
src/engine/mod.rs          add element_contained_by, multirange_union
src/engine/flags.rs        no change
src/engine/canonical.rs    no change
src/multirange_types.rs    add typed wrappers for new ops
src/func/predicates.rs     add multirange_contains_element, multirange_isempty
src/func/extract.rs        add multirange_upper/upper_inc/bounds
src/func/setops.rs         add multirange_union, multirange_length
src/lib.rs                 register new VDF impls
tests/multirange_gaps.rs   unit tests for new ops
tests/proptest_suite.rs    proptests for element containment and union inverse
examples/booking_multirange_demo.rs extend with element containment + union
README.md                  document new surface
CHANGELOG.md               unreleased entry update
```

## Test/CI gates

- `cargo test --workspace` must stay green
- `cargo clippy --all-targets --all-features -- -D warnings` must stay green
- `cargo llvm-cov --all-targets --ignore-filename-regex 'vendor/|villagesql/' --fail-under-lines 50`
- mysql-test expansion for each new SQL function, with `.result` regeneration via `perl mysql-test-run.pl --record`

## Sequencing recommendation

1. Slice 1a — element containment. Highest value, replaces the `NOT EXISTS + OVERLAPS` pattern directly.
2. Slice 1b — multirange UNION. Makes schedules additive.
3. Slice 2 — accessors/predicates. Completes the type symmetry with single-range.
4. Slice 3 — range_agg/unnest. Only if SDK exposes stateful aggregates.
5. Slice 4 — exclusion primitive. Only if there is a server-side constraint path.
