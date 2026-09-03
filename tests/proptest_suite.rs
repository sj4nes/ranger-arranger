// Property + differential tests via `proptest`.
//
// Differential oracle: an INDEPENDENT reference implementation over concrete
// integer sets (brute-force `BTreeSet`) within a bounded domain. The engine's
// algebra (intersect/merge/difference/contains/overlaps/adjacent) must agree
// with set theory. This is a genuine second implementation, not a re-call of
// the engine.

use proptest::prelude::*;
use std::collections::BTreeSet;
use vsql_ranger_arranger::engine::{
    Range, adjacent, canonicalize, contains_point, contains_range, difference, intersect, merge,
    overlaps,
};

const BOUND: i64 = 100;

/// The funcs always feed CANONICAL `[)` ranges to the engine (encode canonicalizes).
/// Mirror that: canonicalize each generated range through a discrete subtype before
/// exercising the algebra. This keeps the engine's precondition honest while the
/// set-based reference stays the ground truth.
fn canon(r: &Range) -> Range {
    canonicalize::<vsql_ranger_arranger::subtype::int8::Int8Ops>(r)
}
fn arb_finite() -> impl Strategy<Value = Range> {
    // Finite discrete range with endpoints inside [-BOUND, BOUND], lower <= upper.
    (
        -BOUND..=BOUND,
        -BOUND..=BOUND,
        prop::bool::ANY,
        prop::bool::ANY,
    )
        .prop_filter_map("ordered", |(l, u, li, ui)| {
            let (lower, upper, lower_inc, upper_inc) = if l <= u {
                (l, u, li, ui)
            } else {
                (u, l, ui, li)
            };
            Some(Range {
                empty: false,
                lower_inf: false,
                upper_inf: false,
                lower_inc,
                upper_inc,
                lower: lower as i128,
                upper: upper as i128,
            })
        })
}

fn arb_any() -> impl Strategy<Value = Range> {
    // Any range: finite (above) or empty or infinity-flagged.
    prop_oneof![
        arb_finite(),
        Just(Range::empty()),
        arb_finite().prop_map(|mut r| {
            r.lower_inf = true;
            r
        }),
        arb_finite().prop_map(|mut r| {
            r.upper_inf = true;
            r
        }),
    ]
}

// ---- Independent reference model (set-based, brute force) ----

/// Expand a FINITE range to the set of contained integer ordinals (discrete `[)` model).
fn to_set(r: &Range) -> BTreeSet<i64> {
    assert!(!r.lower_inf && !r.upper_inf, "to_set needs finite range");
    if r.empty {
        return BTreeSet::new();
    }
    let lo: i64 = if r.lower_inc {
        r.lower as i64
    } else {
        r.lower as i64 + 1
    };
    let hi: i64 = if r.upper_inc {
        r.upper as i64
    } else {
        r.upper as i64 - 1
    };
    let mut s = BTreeSet::new();
    if lo <= hi {
        for x in lo..=hi {
            s.insert(x);
        }
    }
    s
}

/// Canonicalize a set back to a discrete `[)` Range.
fn set_to_range(s: &BTreeSet<i64>) -> Range {
    if s.is_empty() {
        return Range::empty();
    }
    let min = *s.iter().next().unwrap();
    let max = *s.iter().next_back().unwrap();
    Range {
        empty: false,
        lower_inf: false,
        upper_inf: false,
        lower_inc: true,
        upper_inc: false,
        lower: min as i128,
        upper: (max as i128) + 1, // exclusive upper in `[)`
    }
}

// Independent ordinal-based reference for contains/overlaps (works on infinite ranges).
fn ref_max(r: &Range) -> Option<i128> {
    if r.empty || r.upper_inf {
        None
    } else if r.upper_inc {
        Some(r.upper)
    } else {
        Some(r.upper - 1)
    }
}
fn ref_min(r: &Range) -> Option<i128> {
    if r.empty || r.lower_inf {
        None
    } else if r.lower_inc {
        Some(r.lower)
    } else {
        Some(r.lower + 1)
    }
}
fn ref_contains(r: &Range, p: i128) -> bool {
    if r.empty {
        return false;
    }
    let lo_ok = r.lower_inf || p >= ref_min(r).unwrap();
    let hi_ok = r.upper_inf || p <= ref_max(r).unwrap();
    lo_ok && hi_ok
}
fn ref_overlaps(a: &Range, b: &Range) -> bool {
    if a.empty || b.empty {
        return false;
    }
    let before = |x: &Range, y: &Range| match (ref_max(x), ref_min(y)) {
        (Some(mx), Some(mn)) => mx < mn,
        _ => false, // an infinite bound means never strictly before
    };
    !before(a, b) && !before(b, a)
}

proptest! {
    // Differential: intersect == set intersection.
    #[test]
    fn prop_intersect_matches_set(r1 in arb_finite(), r2 in arb_finite()) {
        let (a, b) = (canon(&r1), canon(&r2));
        let got = intersect(&a, &b);
        let expected = set_to_range(&to_set(&a).intersection(&to_set(&b)).cloned().collect());
        prop_assert_eq!(got, expected);
    }

    // Differential: merge == set union (for the bounded, always-contiguous-enclosing case).
    #[test]
    fn prop_merge_matches_set(r1 in arb_finite(), r2 in arb_finite()) {
        let (a, b) = (canon(&r1), canon(&r2));
        let got = merge(&a, &b);
        let expected = set_to_range(&to_set(&a).union(&to_set(&b)).cloned().collect());
        prop_assert_eq!(got, expected);
    }

    // Differential: difference == set difference, as 0/1/2 pieces.
    #[test]
    fn prop_difference_matches_set(r1 in arb_finite(), r2 in arb_finite()) {
        let (a, b) = (canon(&r1), canon(&r2));
        let got = difference(&a, &b);
        let expected_set: BTreeSet<i64> = to_set(&a)
            .difference(&to_set(&b))
            .cloned()
            .collect();
        // Recompose expected pieces by splitting the difference set on gaps.
        let expected_pieces = set_to_pieces(&expected_set);
        prop_assert_eq!(got.len(), expected_pieces.len(), "piece count mismatch");
        for (g, e) in got.iter().zip(expected_pieces.iter()) {
            prop_assert_eq!(g, e);
        }
    }

    // Differential (ordinal): contains_point agrees with independent reference.
    #[test]
    fn prop_contains_matches_ref(r in arb_any(), p in -BOUND..=BOUND) {
        let r = canon(&r);
        let engine = contains_point(&r, p as i128);
        prop_assert_eq!(engine, ref_contains(&r, p as i128));
    }

    // Differential (ordinal): overlaps agrees with independent reference.
    #[test]
    fn prop_overlaps_matches_ref(a in arb_any(), b in arb_any()) {
        let (a, b) = (canon(&a), canon(&b));
        prop_assert_eq!(overlaps(&a, &b), ref_overlaps(&a, &b));
    }

    // Differential (ordinal): adjacent agrees with independent reference.
    #[test]
    fn prop_adjacent_matches_ref(a in arb_any(), b in arb_any()) {
        let (a, b) = (canon(&a), canon(&b));
        let engine = adjacent(&a, &b);
        // reference (canonical `[)` bounds: lower inclusive, upper exclusive):
        // adjacent iff disjoint, non-empty, and the exclusive upper of one equals
        // the inclusive lower of the other (no gap, no overlap).
        let refv = !a.empty
            && !b.empty
            && !ref_overlaps(&a, &b)
            && ((!a.upper_inf && !b.lower_inf && a.upper == b.lower)
                || (!b.upper_inf && !a.lower_inf && b.upper == a.lower));
        prop_assert_eq!(engine, refv);
    }

    // contains_range: a contains b iff every element of b's set is in a's set.
    #[test]
    fn prop_contains_range_matches_set(a in arb_finite(), b in arb_finite()) {
        let (a, b) = (canon(&a), canon(&b));
        let engine = contains_range(&a, &b);
        let refv = to_set(&b).is_subset(&to_set(&a));
        prop_assert_eq!(engine, refv);
    }

    // Canonical-form invariant: intersect/merge/difference never produce empty-implied
    // finite ranges with lower > upper.
    #[test]
    fn prop_finite_ranges_well_formed(r1 in arb_finite(), r2 in arb_finite()) {
        let (a, b) = (canon(&r1), canon(&r2));
        for r in [intersect(&a, &b), merge(&a, &b)] {
            if !r.empty && !r.lower_inf && !r.upper_inf {
                prop_assert!(r.lower <= r.upper, "lower > upper after op");
            }
        }
    }
}

/// Split a (possibly gapped) difference set into canonical `[)` pieces, mirroring
/// the engine's anti-lossy guarantee (FR-8.4).
fn set_to_pieces(s: &BTreeSet<i64>) -> Vec<Range> {
    if s.is_empty() {
        return vec![];
    }
    let mut pieces = Vec::new();
    let mut cur_lo: Option<i64> = None;
    let mut prev: Option<i64> = None;
    for &x in s {
        match (cur_lo, prev) {
            (None, _) => cur_lo = Some(x),
            (Some(_), Some(p)) if x == p + 1 => { /* contiguous */ }
            (Some(lo), Some(p)) => {
                pieces.push(piece(lo, p));
                cur_lo = Some(x);
            }
            (Some(_), None) => { /* unreachable: prev always Some after first */ }
        }
        prev = Some(x);
    }
    if let (Some(lo), Some(p)) = (cur_lo, prev) {
        pieces.push(piece(lo, p));
    }
    pieces
}

fn piece(lo: i64, hi: i64) -> Range {
    Range {
        empty: false,
        lower_inf: false,
        upper_inf: false,
        lower_inc: true,
        upper_inc: false,
        lower: lo as i128,
        upper: (hi as i128) + 1,
    }
}
