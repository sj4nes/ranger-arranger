# Epic 5 — Generated-Column Indexing Cookbook

How to get fast range queries (overlap, contains, adjacency, before) on a
`*RANGE` column using a **generated column** plus a normal index. This is the
practical substitute for the trigger-based exclusion constraint we do NOT provide
(see Epic 6 / AD-6).

## The problem

A range like `[1,5)` is stored as a single fixed-width blob (one flag byte +
two endpoint ordinals). The server can compare two such blobs for equality and
order, but it cannot use an ordinary B-tree index to answer "which rows overlap
this range?" efficiently — overlap is not a simple scalar comparison.

PostgreSQL solves this with a **GiST index**. VillageSQL's VEF, in this v0.1
extension, has no custom index-access-method hook. So we use the next best
thing that the server *does* support: a **generated column** that reduces the
range to scalar keys, indexed normally.

## Approach: lower/upper + a covering index

For a discrete range stored in canonical `[)` form, the two endpoints
(`LOWER`, `UPPER`) are scalar ordinals. A query that overlaps a target range
`[a, b)` must satisfy:

    row.lower < b   AND   row.upper > a

Both sides are simple scalar comparisons on the endpoints. If we expose the
endpoints as generated columns and index them, the planner can use the index
to prune non-matching rows.

### Step 1 — create the table with a range column

    CREATE TABLE bookings (
        id        BIGINT PRIMARY KEY,
        slot      INT8RANGE,
        -- generated scalar keys for indexing (canonical [) endpoints)
        slot_lo   BIGINT AS (CAST(INT8RANGE_LOWER(slot) AS BIGINT)) STORED,
        slot_hi   BIGINT AS (CAST(INT8RANGE_UPPER(slot) AS BIGINT)) STORED
    );

`INT8RANGE_LOWER` / `INT8RANGE_UPPER` return the canonical endpoints as strings;
cast them to the integer ordinal for the index. `STORED` makes them physically
written so they can be indexed.

### Step 2 — index the generated columns

    CREATE INDEX ix_bookings_slot ON bookings (slot_lo, slot_hi);

### Step 3 — query with the scalar predicates

    -- find bookings that overlap [100, 200)
    SELECT * FROM bookings
    WHERE slot_lo < 200
      AND slot_hi > 100;

The planner can now use `ix_bookings_slot` to seek into the relevant bands
instead of scanning every row and calling `INT8RANGE_OVERLAPS`.

## Why this is a stand-in, not a full GiST

- It answers **overlap** and **contains** efficiently, but it is a manual
  rewrite of the predicate. The planner does not automatically know that
  `INT8RANGE_OVERLAPS(slot, x)` is equivalent to the scalar pair, so you write
  the scalar form yourself (or wrap it in a VIEW).
- It does **not** enforce that no two rows overlap (that needs a constraint
  exclusion / trigger — out of scope, see Epic 6).
- For temporal types (`DATERANGE`, `DATETIMERANGE`) the same pattern applies:
  generate numeric epoch columns (days / microseconds) and index those.

## Performance note

The set-operation functions (`INT8RANGE_INTERSECT`, `INT8RANGE_MERGE`, …) were
profiled with `cargo bench` (see `benches/engine_bench.rs`). Serializing a
result range directly to bytes — instead of round-tripping it through a literal
string and re-parsing — cut the cost ~38× for dates and ~65× for datetimes
(Epic 7). The generated-column pattern above keeps the hot path on
`LOWER`/`UPPER` (a ~7 ns `to_range` + ~100 ns `decode`), so the index lookup,
not the extension, dominates query time.
