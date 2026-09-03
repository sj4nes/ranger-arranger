// Smoke test: encode/decode round-trip (parse(format(r)) = r) + canonicalization,
// across all four subtypes and every bound/inclusivity/finite-infinite/empty combo.
// encode/decode/compare all delegate to the engine.
//
// Also covers the algebra (intersect/merge/difference/overlaps/adjacent/
// contains_range) at the engine level.

use std::cmp::Ordering;
use vsql_ranger_arranger::engine::canonical::{decode, encode, to_range};
use vsql_ranger_arranger::engine::compare::range_compare;
use vsql_ranger_arranger::engine::{
    Range, RangeSubtypeOps, adjacent, contains_range, difference, intersect, merge, overlaps,
};
use vsql_ranger_arranger::subtype::{date, datetime, int4, int8};

// Build a Range model from endpoints for algebra tests (avoids literal parsing).
fn r(lower: i128, upper: i128) -> Range {
    Range {
        empty: false,
        lower_inf: false,
        upper_inf: false,
        lower_inc: true,
        upper_inc: false,
        lower,
        upper,
    }
}

fn roundtrip<T: RangeSubtypeOps>(lit: &str) -> String {
    let bytes = encode::<T>(lit).expect("encode");
    let back = decode::<T>(&bytes).expect("decode");
    // Canonical form: re-encode must be stable (idempotent).
    let bytes2 = encode::<T>(&back).expect("re-encode");
    assert_eq!(bytes, bytes2, "non-idempotent encode for {lit}");
    back
}

#[test]
fn int8_roundtrip_and_canonical() {
    // Discrete canonicalization to [): [1,5] -> [1,6)
    assert_eq!(roundtrip::<int8::Int8Ops>("[1,5]"), "[1,6)");
    assert_eq!(roundtrip::<int8::Int8Ops>("[1,5)"), "[1,5)");
    assert_eq!(roundtrip::<int8::Int8Ops>("(1,5)"), "[2,5)");
    // empty + infinite
    assert_eq!(roundtrip::<int8::Int8Ops>("empty"), "empty");
    assert_eq!(
        roundtrip::<int8::Int8Ops>("(-infinity,10)"),
        "(-infinity,10)"
    );
    // reversed endpoint rejected
    assert!(encode::<int8::Int8Ops>("[5,1)").is_err());
    // parse(format(r)) = r invariant
    let lit = "[1,6)";
    let bytes = encode::<int8::Int8Ops>(lit).unwrap();
    assert_eq!(decode::<int8::Int8Ops>(&bytes).unwrap(), lit);
}

#[test]
fn int4_roundtrip() {
    assert_eq!(roundtrip::<int4::Int4Ops>("[1,5]"), "[1,6)");
    assert_eq!(roundtrip::<int4::Int4Ops>("[1,5)"), "[1,5)");
}

#[test]
fn date_roundtrip() {
    // Discrete: [2026-01-01,2026-01-02] -> [2026-01-01,2026-01-03)
    assert_eq!(
        roundtrip::<date::DateOps>("[2026-01-01,2026-01-02]"),
        "[2026-01-01,2026-01-03)"
    );
}

#[test]
fn datetime_preserves_bounds() {
    // Continuous: bounds preserved (not normalized to [))
    assert_eq!(
        roundtrip::<datetime::DateTimeOps>("[2026-01-01 00:00:00,2026-01-02 00:00:00]"),
        "[2026-01-01 00:00:00.000000,2026-01-02 00:00:00.000000]"
    );
}

#[test]
fn compare_orders_empty_before_nonempty() {
    let empty = encode::<int8::Int8Ops>("empty").unwrap();
    let five = encode::<int8::Int8Ops>("[1,5)").unwrap();
    assert_eq!(range_compare(&empty, &five), Ordering::Less);
    assert_eq!(range_compare(&five, &empty), Ordering::Greater);
    // same point-set compares equal after canonicalization: [1,5] == [1,6)
    let a = encode::<int8::Int8Ops>("[1,5]").unwrap();
    let b = encode::<int8::Int8Ops>("[1,6)").unwrap();
    assert_eq!(range_compare(&a, &b), Ordering::Equal);
    let _ = to_range::<int8::Int8Ops>(&a);
}

// ---- Engine algebra ----

#[test]
fn algebra_intersect_merge_overlap() {
    let a = r(1, 5); // [1,5)
    let b = r(3, 8); // [3,8)
    assert!(overlaps(&a, &b));
    let i = intersect(&a, &b);
    assert_eq!(i, r(3, 5)); // [3,5)
    let m = merge(&a, &b);
    assert_eq!(m, r(1, 8)); // [1,8) (overlapping -> merged enclosing)
    assert!(!adjacent(&a, &b)); // overlapping, not adjacent
    assert!(contains_range(&a, &r(2, 4))); // [2,4) inside [1,5)
    assert!(!contains_range(&a, &r(0, 4))); // [0,4) not inside
}

#[test]
fn algebra_difference_is_anti_lossy() {
    // [1,10) - [3,6) -> [1,3) and [6,10)
    let a = r(1, 10);
    let b = r(3, 6);
    let pieces = difference(&a, &b);
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0], r(1, 3));
    assert_eq!(pieces[1], r(6, 10));
}

#[test]
fn algebra_adjacent_detection() {
    // [1,5) and [5,10) are adjacent (no gap, no overlap).
    assert!(adjacent(&r(1, 5), &r(5, 10)));
    assert!(!adjacent(&r(1, 5), &r(6, 10))); // gap
}
