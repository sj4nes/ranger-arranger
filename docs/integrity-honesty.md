# Epic 6 — Integrity & Honesty Boundaries

What this extension guarantees, and — just as important — what it does NOT. A
range type is only useful if you can trust it under concurrent writes and
overlapping schedules. This document is the honesty boundary (PRD §11 / AD-6):
it states, in plain terms, where the guarantees stop.

## What it does guarantee

- **Correct algebra.** Every set operation (intersect, merge, difference,
  overlaps, contains, adjacent, before, equals) is checked by an independent
  differential oracle in `tests/proptest_suite.rs` (a `BTreeSet` model over a
  bounded domain) across tens of thousands of randomized ranges, including empty
  and infinite ones. The algebra is also exercised live against a real
  VillageSQL server (`mysql-test/`).
- **Canonical storage.** Ranges are stored in one canonical form (`[)`, lower
  inclusive / upper exclusive for discrete types). Two ways of writing the same
  set — `[1,5]` and `[1,6)` — compare and store identically. This is verified
  by the `canonical_eq` test and the `INT8RANGE_EQUALS` live check.
- **Lossless difference.** `INT8RANGE_DIFFERENCE` never collapses a disjoint
  result into one lossy range; it returns a JSON array of pieces
  (`[[1,3),[6,10)]`). No information is dropped.
- **NULL-safe.** Every function returns `NULL` when any input is `NULL`, never
  a wrong range. Verified in the func-ABI and live suites.
- **No overlap of the *model*.** The engine's comparison and canonicalization
  are owned in one place (AD-1), so a subtype cannot quietly diverge from the
  contract.

## What it does NOT guarantee (the boundary)

1. **No trigger-based exclusion constraint.** PostgreSQL's `EXCLUDE USING
   gist (slot WITH &&)` stops two rows from overlapping at write time. VEF's
   `custom_type!` in this v0.1 has **no `EXCLUDE` hook**. Nothing in this
   extension prevents you from inserting two `INT8RANGE` rows that overlap.
   If you need that guarantee today, you must enforce it at the application
   layer (or via the generated-column index pattern in Epic 5 plus a check
   query before commit).

2. **No concurrent-write integrity.** Two transactions inserting overlapping
   ranges can both succeed; the extension does not take locks or run a
   uniqueness/exclusion check across rows. Correctness under concurrency is the
   server's job, and the server does not yet offer the hook this extension
   would need.

3. **`RANGE_ASSERT_AVAILABLE` availability windows are deferred.** The PRD
   describes Levels 3–5: declarative "no two bookings overlap this room"
   constraints, auto-resolution, and audit trails. These are **not implemented**
   in v0.1. What exists is the primitive algebra plus the indexing cookbook.
   Claiming more would be dishonest.

4. **Temporal precision is the subtype's.** `DATETIMERANGE` stores microseconds.
   If your source data has nanosecond precision, it is truncated on write. This
   is a deliberate scope choice (fixed-width endpoints), not a bug, but you
   should know it.

5. **`UNION` of disjoint ranges is an error by design.** `INT8RANGE_UNION`
   requires the inputs to overlap (it is the set union of overlapping ranges).
   For a spanning interval over a gap, use `INT8RANGE_MERGE`, which returns the
   minimal enclosing range — accepting that the gap is included. We deliberately
   do not silently invent points inside the gap.

## Why this matters for a work sample

An interviewer should be able to trust the claims. The tests prove the algebra;
the live server run proves the integration; this document proves we know exactly
where the safety ends. Shipping a range type that *looks* like it enforces
non-overlap but does not would be the dangerous outcome — so the boundary is
stated up front, in the README and here.
