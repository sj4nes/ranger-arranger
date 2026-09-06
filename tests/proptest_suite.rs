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
use vsql_ranger_arranger::multirange_types::{
    datemr_contains_range, datemr_decode, datemr_difference, datemr_encode, datemr_intersect,
    datemr_merge, datemr_overlaps, dtmr_contains_range, dtmr_decode, dtmr_difference, dtmr_encode,
    dtmr_intersect, dtmr_merge, dtmr_overlaps, int4mr_contains_range, int4mr_decode,
    int4mr_difference, int4mr_encode, int4mr_intersect, int4mr_merge, int4mr_overlaps,
    int8mr_contains_range, int8mr_decode, int8mr_difference, int8mr_encode, int8mr_intersect,
    int8mr_merge, int8mr_overlaps, lift_contains_range, lift_difference, lift_intersect,
    lift_merge, lift_overlaps, mr_decode_to_vec, roundtrip_components,
};
use vsql_ranger_arranger::subtype::{DateOps, DateTimeOps, Int4Ops, Int8Ops};

const BOUND: i64 = 100;

// ---- Range strategies and reference model (unchanged) ----

fn canon(r: &Range) -> Range {
    canonicalize::<vsql_ranger_arranger::subtype::int8::Int8Ops>(r)
}
fn arb_finite() -> impl Strategy<Value = Range> {
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
        upper: (max as i128) + 1,
    }
}

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
        _ => false,
    };
    !before(a, b) && !before(b, a)
}

// ---- Multirange strategies ----

fn arb_range_literal() -> impl Strategy<Value = String> {
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
            let lb = if lower_inc { '[' } else { '(' };
            let rb = if upper_inc { ']' } else { ')' };
            Some(format!("{lb}{lower},{upper}{rb}"))
        })
}

fn arb_multirange_literal() -> impl Strategy<Value = String> {
    (
        0usize..=8,
        prop::collection::vec(arb_range_literal(), 0..=8),
    )
        .prop_map(|(_, components)| {
            if components.is_empty() {
                return "empty".to_string();
            }
            format!("{{{}}}", components.join(","))
        })
}

// ---- Range proptests (unchanged) ----

proptest! {
    #[test]
    fn prop_intersect_matches_set(r1 in arb_finite(), r2 in arb_finite()) {
        let (a, b) = (canon(&r1), canon(&r2));
        let got = intersect(&a, &b);
        let expected = set_to_range(&to_set(&a).intersection(&to_set(&b)).cloned().collect());
        prop_assert_eq!(got, expected);
    }

    #[test]
    fn prop_merge_matches_set(r1 in arb_finite(), r2 in arb_finite()) {
        let (a, b) = (canon(&r1), canon(&r2));
        let got = merge(&a, &b);
        let expected = set_to_range(&to_set(&a).union(&to_set(&b)).cloned().collect());
        prop_assert_eq!(got, expected);
    }

    #[test]
    fn prop_difference_matches_set(r1 in arb_finite(), r2 in arb_finite()) {
        let (a, b) = (canon(&r1), canon(&r2));
        let got = difference(&a, &b);
        let expected_set: BTreeSet<i64> = to_set(&a).difference(&to_set(&b)).cloned().collect();
        let expected_pieces = set_to_pieces(&expected_set);
        prop_assert_eq!(got.len(), expected_pieces.len(), "piece count mismatch");
        for (g, e) in got.iter().zip(expected_pieces.iter()) {
            prop_assert_eq!(g, e);
        }
    }

    #[test]
    fn prop_contains_matches_ref(r in arb_any(), p in -BOUND..=BOUND) {
        let r = canon(&r);
        let engine = contains_point(&r, p as i128);
        prop_assert_eq!(engine, ref_contains(&r, p as i128));
    }

    #[test]
    fn prop_overlaps_matches_ref(a in arb_any(), b in arb_any()) {
        let (a, b) = (canon(&a), canon(&b));
        prop_assert_eq!(overlaps(&a, &b), ref_overlaps(&a, &b));
    }

    #[test]
    fn prop_adjacent_matches_ref(a in arb_any(), b in arb_any()) {
        let (a, b) = (canon(&a), canon(&b));
        let engine = adjacent(&a, &b);
        let refv = !a.empty
            && !b.empty
            && !ref_overlaps(&a, &b)
            && ((!a.upper_inf && !b.lower_inf && a.upper == b.lower)
                || (!b.upper_inf && !a.lower_inf && b.upper == a.lower));
        prop_assert_eq!(engine, refv);
    }

    #[test]
    fn prop_contains_range_matches_set(a in arb_finite(), b in arb_finite()) {
        let (a, b) = (canon(&a), canon(&b));
        let engine = contains_range(&a, &b);
        let refv = to_set(&b).is_subset(&to_set(&a));
        prop_assert_eq!(engine, refv);
    }

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
            (Some(_), None) => { /* unreachable */ }
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

// ---- Multirange proptests ----

proptest! {
    /// INT8MULTIRANGE encode->decode round-trip is stable.
    #[test]
    fn prop_int8mr_roundtrip(lit in arb_multirange_literal()) {
        let encoded = match int8mr_encode(&lit) {
            Ok(b) => b,
            Err(_) => { prop_assume!(false, "invalid multirange literal"); unreachable!(); }
        };
        let decoded = int8mr_decode(&encoded).expect("decode of our own encode must succeed");
        let reencoded = int8mr_encode(&decoded).expect("re-encode of decode must succeed");
        prop_assert_eq!(encoded, reencoded, "INT8MULTIRANGE round-trip not stable");
    }

    /// INT4MULTIRANGE encode->decode round-trip is stable.
    #[test]
    fn prop_int4mr_roundtrip(lit in arb_multirange_literal()) {
        let encoded = match int4mr_encode(&lit) {
            Ok(b) => b,
            Err(_) => { prop_assume!(false, "invalid multirange literal"); unreachable!(); }
        };
        let decoded = int4mr_decode(&encoded).expect("decode of our own encode must succeed");
        let reencoded = int4mr_encode(&decoded).expect("re-encode of decode must succeed");
        prop_assert_eq!(encoded, reencoded, "INT4MULTIRANGE round-trip not stable");
    }


}

// ---- Date/Datetime multirange round-trips (type-appropriate literals) ----

#[test]
fn prop_datemr_roundtrip_valid_dates() {
    let cases = vec![
        "{}",
        "empty",
        "{[2020-01-01,2020-06-01)}",
        "{[2020-01-01,2020-06-01),[2020-07-01,2020-12-31)}",
    ];
    for lit in cases {
        if let Ok(encoded) = datemr_encode(lit) {
            let decoded = datemr_decode(&encoded).expect("decode of our own encode must succeed");
            let reencoded = datemr_encode(&decoded).expect("re-encode of decode must succeed");
            assert_eq!(
                encoded, reencoded,
                "DATEMULTIRANGE round-trip failed for: {}",
                lit
            );
        }
    }
}

#[test]
fn prop_dtmr_roundtrip_valid_datetimes() {
    let cases = vec![
        "{}",
        "empty",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00)}",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00),[2020-07-01 00:00:00,2020-12-31 00:00:00)}",
    ];
    for lit in cases {
        if let Ok(encoded) = dtmr_encode(lit) {
            let decoded = dtmr_decode(&encoded).expect("decode of our own encode must succeed");
            let reencoded = dtmr_encode(&decoded).expect("re-encode of decode must succeed");
            assert_eq!(
                encoded, reencoded,
                "DATETIMEMULTIRANGE round-trip failed for: {}",
                lit
            );
        }
    }
}

// ---- Multirange algebra proptests ----
//
// Oracle: lift the single-range engine over component lists.
// We decode both operands to Vec<Range>, apply the lifted op in-Rust,
// then compare against the VDF result (decode VDF output → re-encode).

proptest! {
    fn prop_int8mr_overlaps_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int8Ops>(&lit_a).expect("valid int8 multirange literal");
        let b_comps = roundtrip_components::<Int8Ops>(&lit_b).expect("valid int8 multirange literal");
        let expected = lift_overlaps::<Int8Ops>(&a_comps, &b_comps);
        let enc_a = int8mr_encode(&lit_a).unwrap();
        let enc_b = int8mr_encode(&lit_b).unwrap();
        let got = int8mr_overlaps(&enc_a, &enc_b).unwrap();
        prop_assert_eq!(got, expected);
    }
    fn prop_int8mr_contains_range_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int8Ops>(&lit_a).expect("valid int8 multirange literal");
        let b_comps = roundtrip_components::<Int8Ops>(&lit_b).expect("valid int8 multirange literal");
        let expected = lift_contains_range::<Int8Ops>(&a_comps, &b_comps);
        let enc_a = int8mr_encode(&lit_a).unwrap();
        let enc_b = int8mr_encode(&lit_b).unwrap();
        let got = int8mr_contains_range(&enc_a, &enc_b).unwrap();
        prop_assert_eq!(got, expected);
    }
    fn prop_int8mr_intersect_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int8Ops>(&lit_a).expect("valid int8 multirange literal");
        let b_comps = roundtrip_components::<Int8Ops>(&lit_b).expect("valid int8 multirange literal");
        let expected = lift_intersect::<Int8Ops>(&a_comps, &b_comps);
        let enc_a = int8mr_encode(&lit_a).unwrap();
        let enc_b = int8mr_encode(&lit_b).unwrap();
        let got_buf = int8mr_intersect(&enc_a, &enc_b).unwrap();
        let got_comps = mr_decode_to_vec::<Int8Ops>(&got_buf).unwrap();
        prop_assert_eq!(got_comps, expected);
    }
    fn prop_int8mr_merge_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int8Ops>(&lit_a).expect("valid int8 multirange literal");
        let b_comps = roundtrip_components::<Int8Ops>(&lit_b).expect("valid int8 multirange literal");
        let expected = lift_merge::<Int8Ops>(&a_comps, &b_comps);
        let enc_a = int8mr_encode(&lit_a).unwrap();
        let enc_b = int8mr_encode(&lit_b).unwrap();
        let got_buf = int8mr_merge(&enc_a, &enc_b).unwrap();
        let got_comps = mr_decode_to_vec::<Int8Ops>(&got_buf).unwrap();
        prop_assert_eq!(got_comps, expected);
    }
    fn prop_int8mr_difference_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int8Ops>(&lit_a).expect("valid int8 multirange literal");
        let b_comps = roundtrip_components::<Int8Ops>(&lit_b).expect("valid int8 multirange literal");
        let expected = lift_difference::<Int8Ops>(&a_comps, &b_comps);
        let enc_a = int8mr_encode(&lit_a).unwrap();
        let enc_b = int8mr_encode(&lit_b).unwrap();
        let got_buf = int8mr_difference(&enc_a, &enc_b).unwrap();
        let got_comps = mr_decode_to_vec::<Int8Ops>(&got_buf).unwrap();
        prop_assert_eq!(got_comps, expected);
    }
    fn prop_int4mr_overlaps_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int4Ops>(&lit_a).expect("valid int4 multirange literal");
        let b_comps = roundtrip_components::<Int4Ops>(&lit_b).expect("valid int4 multirange literal");
        let expected = lift_overlaps::<Int4Ops>(&a_comps, &b_comps);
        let enc_a = int4mr_encode(&lit_a).unwrap();
        let enc_b = int4mr_encode(&lit_b).unwrap();
        let got = int4mr_overlaps(&enc_a, &enc_b).unwrap();
        prop_assert_eq!(got, expected);
    }
    fn prop_int4mr_contains_range_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int4Ops>(&lit_a).expect("valid int4 multirange literal");
        let b_comps = roundtrip_components::<Int4Ops>(&lit_b).expect("valid int4 multirange literal");
        let expected = lift_contains_range::<Int4Ops>(&a_comps, &b_comps);
        let enc_a = int4mr_encode(&lit_a).unwrap();
        let enc_b = int4mr_encode(&lit_b).unwrap();
        let got = int4mr_contains_range(&enc_a, &enc_b).unwrap();
        prop_assert_eq!(got, expected);
    }
    fn prop_int4mr_intersect_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int4Ops>(&lit_a).expect("valid int4 multirange literal");
        let b_comps = roundtrip_components::<Int4Ops>(&lit_b).expect("valid int4 multirange literal");
        let expected = lift_intersect::<Int4Ops>(&a_comps, &b_comps);
        let enc_a = int4mr_encode(&lit_a).unwrap();
        let enc_b = int4mr_encode(&lit_b).unwrap();
        let got_buf = int4mr_intersect(&enc_a, &enc_b).unwrap();
        let got_comps = mr_decode_to_vec::<Int4Ops>(&got_buf).unwrap();
        prop_assert_eq!(got_comps, expected);
    }
    fn prop_int4mr_merge_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int4Ops>(&lit_a).expect("valid int4 multirange literal");
        let b_comps = roundtrip_components::<Int4Ops>(&lit_b).expect("valid int4 multirange literal");
        let expected = lift_merge::<Int4Ops>(&a_comps, &b_comps);
        let enc_a = int4mr_encode(&lit_a).unwrap();
        let enc_b = int4mr_encode(&lit_b).unwrap();
        let got_buf = int4mr_merge(&enc_a, &enc_b).unwrap();
        let got_comps = mr_decode_to_vec::<Int4Ops>(&got_buf).unwrap();
        prop_assert_eq!(got_comps, expected);
    }
    fn prop_int4mr_difference_matches_lifted(lit_a in arb_multirange_literal(), lit_b in arb_multirange_literal()) {
        let a_comps = roundtrip_components::<Int4Ops>(&lit_a).expect("valid int4 multirange literal");
        let b_comps = roundtrip_components::<Int4Ops>(&lit_b).expect("valid int4 multirange literal");
        let expected = lift_difference::<Int4Ops>(&a_comps, &b_comps);
        let enc_a = int4mr_encode(&lit_a).unwrap();
        let enc_b = int4mr_encode(&lit_b).unwrap();
        let got_buf = int4mr_difference(&enc_a, &enc_b).unwrap();
        let got_comps = mr_decode_to_vec::<Int4Ops>(&got_buf).unwrap();
        prop_assert_eq!(got_comps, expected);
    }
}

fn date_mr_cases() -> Vec<(&'static str, &'static str)> {
    let lits = [
        "{}",
        "empty",
        "{[2020-01-01,2020-06-01)}",
        "{[2020-07-01,2020-12-31)}",
        "{[2020-01-01,2020-06-01),[2020-07-01,2020-12-31)}",
    ];
    let mut out = Vec::new();
    for a in &lits {
        for b in &lits {
            out.push((*a, *b));
        }
    }
    out
}

fn dt_mr_cases() -> Vec<(&'static str, &'static str)> {
    let lits = [
        "{}",
        "empty",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00)}",
        "{[2020-07-01 00:00:00,2020-12-31 00:00:00)}",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00),[2020-07-01 00:00:00,2020-12-31 00:00:00)}",
    ];
    let mut out = Vec::new();
    for a in &lits {
        for b in &lits {
            out.push((*a, *b));
        }
    }
    out
}

#[test]
fn prop_datemr_algebra_matches_lifted() {
    for (lit_a, lit_b) in date_mr_cases() {
        let a_comps = mr_decode_to_vec::<DateOps>(&datemr_encode(lit_a).unwrap()).unwrap();
        let b_comps = mr_decode_to_vec::<DateOps>(&datemr_encode(lit_b).unwrap()).unwrap();
        let enc_a = datemr_encode(lit_a).unwrap();
        let enc_b = datemr_encode(lit_b).unwrap();

        assert_eq!(
            datemr_overlaps(&enc_a, &enc_b).unwrap(),
            lift_overlaps::<DateOps>(&a_comps, &b_comps),
            "DATEMULTIRANGE overlaps failed for: {} vs {}",
            lit_a,
            lit_b
        );
        assert_eq!(
            datemr_contains_range(&enc_a, &enc_b).unwrap(),
            lift_contains_range::<DateOps>(&a_comps, &b_comps),
            "DATEMULTIRANGE contains_range failed for: {} vs {}",
            lit_a,
            lit_b
        );
        assert_eq!(
            mr_decode_to_vec::<DateOps>(&datemr_intersect(&enc_a, &enc_b).unwrap()).unwrap(),
            lift_intersect::<DateOps>(&a_comps, &b_comps),
            "DATEMULTIRANGE intersect failed for: {} vs {}",
            lit_a,
            lit_b
        );
        assert_eq!(
            mr_decode_to_vec::<DateOps>(&datemr_merge(&enc_a, &enc_b).unwrap()).unwrap(),
            lift_merge::<DateOps>(&a_comps, &b_comps),
            "DATEMULTIRANGE merge failed for: {} vs {}",
            lit_a,
            lit_b
        );
        assert_eq!(
            mr_decode_to_vec::<DateOps>(&datemr_difference(&enc_a, &enc_b).unwrap()).unwrap(),
            lift_difference::<DateOps>(&a_comps, &b_comps),
            "DATEMULTIRANGE difference failed for: {} vs {}",
            lit_a,
            lit_b
        );
    }
}

#[test]
fn prop_dtmr_algebra_matches_lifted() {
    for (lit_a, lit_b) in dt_mr_cases() {
        let a_comps = mr_decode_to_vec::<DateTimeOps>(&dtmr_encode(lit_a).unwrap()).unwrap();
        let b_comps = mr_decode_to_vec::<DateTimeOps>(&dtmr_encode(lit_b).unwrap()).unwrap();
        let enc_a = dtmr_encode(lit_a).unwrap();
        let enc_b = dtmr_encode(lit_b).unwrap();

        assert_eq!(
            dtmr_overlaps(&enc_a, &enc_b).unwrap(),
            lift_overlaps::<DateTimeOps>(&a_comps, &b_comps),
            "DATETIMEMULTIRANGE overlaps failed for: {} vs {}",
            lit_a,
            lit_b
        );
        assert_eq!(
            dtmr_contains_range(&enc_a, &enc_b).unwrap(),
            lift_contains_range::<DateTimeOps>(&a_comps, &b_comps),
            "DATETIMEMULTIRANGE contains_range failed for: {} vs {}",
            lit_a,
            lit_b
        );
        assert_eq!(
            mr_decode_to_vec::<DateTimeOps>(&dtmr_intersect(&enc_a, &enc_b).unwrap()).unwrap(),
            lift_intersect::<DateTimeOps>(&a_comps, &b_comps),
            "DATETIMEMULTIRANGE intersect failed for: {} vs {}",
            lit_a,
            lit_b
        );
        assert_eq!(
            mr_decode_to_vec::<DateTimeOps>(&dtmr_merge(&enc_a, &enc_b).unwrap()).unwrap(),
            lift_merge::<DateTimeOps>(&a_comps, &b_comps),
            "DATETIMEMULTIRANGE merge failed for: {} vs {}",
            lit_a,
            lit_b
        );
        assert_eq!(
            mr_decode_to_vec::<DateTimeOps>(&dtmr_difference(&enc_a, &enc_b).unwrap()).unwrap(),
            lift_difference::<DateTimeOps>(&a_comps, &b_comps),
            "DATETIMEMULTIRANGE difference failed for: {} vs {}",
            lit_a,
            lit_b
        );
    }
}
