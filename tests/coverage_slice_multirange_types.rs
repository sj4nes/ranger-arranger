// Coverage slice: multirange_types.rs.
// Exercises mr_hash, error paths in parse/decode/compare, normalize edge cases,
// and mr_encode_components overflow.

use vsql_ranger_arranger::engine::Range;
use vsql_ranger_arranger::multirange_types::{
    datemr_compare, datemr_encode, dtmr_compare, dtmr_encode, int4mr_compare, int4mr_encode,
    int8mr_compare, int8mr_decode, int8mr_encode, mr_canonical_empty, mr_decode_to_vec,
    mr_encode_components, mr_hash, mr_merge_buffers, normalize_components, roundtrip_components,
};
use vsql_ranger_arranger::subtype::{DateOps, DateTimeOps, Int4Ops, Int8Ops};

// ── mr_hash ────────────────────────────────────────────────────────────────

#[test]
fn cov_mr_hash_empty() {
    let empty = mr_canonical_empty::<Int8Ops>();
    let h = mr_hash(&empty);
    // Just verify it doesn't panic and produces a consistent value
    assert_eq!(h, mr_hash(&empty));
}

#[test]
fn cov_mr_hash_nonempty() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[1,5)}").unwrap();
    assert_eq!(mr_hash(&a), mr_hash(&b));
}

#[test]
fn cov_mr_hash_different() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[10,15)}").unwrap();
    // Hashes should differ (not guaranteed but extremely likely)
    assert_ne!(mr_hash(&a), mr_hash(&b));
}

// ── parse_multirange_literal error paths ───────────────────────────────────

#[test]
fn cov_parse_mr_unclosed_range() {
    let result = int8mr_encode("{[1,5)");
    assert!(result.is_err());
}

#[test]
fn cov_parse_mr_missing_brace() {
    let result = int8mr_encode("[1,5)");
    assert!(result.is_err());
}

#[test]
fn cov_parse_mr_invalid_char() {
    let result = int8mr_encode("{[1,5),x}");
    assert!(result.is_err());
}

// ── mr_decode_to_vec error paths ──────────────────────────────────────────

#[test]
fn cov_mr_decode_wrong_length() {
    let result = mr_decode_to_vec::<Int8Ops>(&[1, 2, 3]);
    assert!(result.is_err());
}

#[test]
fn cov_mr_decode_count_exceeds_max() {
    // Create a buffer with count > MAX_COMPONENTS (256)
    let mut buf = vec![0u8; 4354];
    buf[0] = 0x01;
    buf[1] = 0x01; // count = 257
    let result = mr_decode_to_vec::<Int8Ops>(&buf);
    assert!(result.is_err());
}

// ── mr_compare_inner error paths ──────────────────────────────────────────

#[test]
fn cov_int8mr_compare_different_length() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let mut b = a.clone();
    b.push(0);
    // Should not panic, just compare by length
    let _ = int8mr_compare(&a, &b);
}

#[test]
fn cov_int8mr_compare_wrong_length() {
    let a = vec![0u8; 100];
    let b = vec![0u8; 100];
    // Both wrong length but same, should return Equal
    assert_eq!(int8mr_compare(&a, &b), std::cmp::Ordering::Equal);
}

// ── normalize_components edge cases ────────────────────────────────────────

#[test]
fn cov_normalize_empty() {
    let result = normalize_components::<Int8Ops>(vec![]).unwrap();
    assert!(result.is_empty());
}

#[test]
fn cov_normalize_all_empty_components() {
    // All components are empty ranges
    let components = vec![Range {
        empty: true,
        lower_inf: false,
        upper_inf: false,
        lower_inc: false,
        upper_inc: false,
        lower: 0,
        upper: 0,
    }];
    let result = normalize_components::<Int8Ops>(components).unwrap();
    assert!(result.is_empty());
}

#[test]
fn cov_normalize_unsorted() {
    // Components out of order
    let a = int8mr_encode("{[10,15),[1,5)}").unwrap();
    let decoded = int8mr_decode(&a).unwrap();
    // Should be normalized to {[1,5),[10,15)}
    assert_eq!(decoded, "{[1,5),[10,15)}");
}

#[test]
fn cov_normalize_adjacent_merge() {
    // Adjacent ranges should merge
    let a = int8mr_encode("{[1,5),[5,10)}").unwrap();
    let decoded = int8mr_decode(&a).unwrap();
    assert_eq!(decoded, "{[1,10)}");
}

#[test]
fn cov_normalize_overlapping_merge() {
    // Overlapping ranges should merge
    let a = int8mr_encode("{[1,7),[3,10)}").unwrap();
    let decoded = int8mr_decode(&a).unwrap();
    assert_eq!(decoded, "{[1,10)}");
}

// ── mr_encode_components overflow ──────────────────────────────────────────

#[test]
fn cov_mr_encode_components_too_many() {
    // Create 257 components (MAX is 256)
    let components = vec![
        Range {
            empty: false,
            lower_inf: false,
            upper_inf: false,
            lower_inc: true,
            upper_inc: false,
            lower: 0,
            upper: 1
        };
        257
    ];
    let result = mr_encode_components::<Int8Ops>(&components);
    assert!(result.is_err());
}

// ── mr_canonical_empty ────────────────────────────────────────────────────

#[test]
fn cov_mr_canonical_empty_int8() {
    let empty = mr_canonical_empty::<Int8Ops>();
    assert_eq!(empty.len(), 4354);
    assert!(empty.iter().all(|&b| b == 0));
}

#[test]
fn cov_mr_canonical_empty_int4() {
    let empty = mr_canonical_empty::<Int4Ops>();
    assert_eq!(empty.len(), 2306);
}

#[test]
fn cov_mr_canonical_empty_date() {
    let empty = mr_canonical_empty::<DateOps>();
    assert_eq!(empty.len(), 4354);
}

#[test]
fn cov_mr_canonical_empty_datetime() {
    let empty = mr_canonical_empty::<DateTimeOps>();
    assert_eq!(empty.len(), 4354);
}

// ── roundtrip_components ──────────────────────────────────────────────────

#[test]
fn cov_roundtrip_components_int8() {
    let comps = roundtrip_components::<Int8Ops>("{[1,5),[10,15)}").unwrap();
    assert_eq!(comps.len(), 2);
    assert_eq!(comps[0].lower, 1);
    assert_eq!(comps[0].upper, 5);
    assert_eq!(comps[1].lower, 10);
    assert_eq!(comps[1].upper, 15);
}

#[test]
fn cov_roundtrip_components_empty() {
    let comps = roundtrip_components::<Int8Ops>("empty").unwrap();
    assert!(comps.is_empty());
}

#[test]
fn cov_roundtrip_components_date() {
    let comps = roundtrip_components::<DateOps>("{[2026-01-01,2026-06-30)}").unwrap();
    assert_eq!(comps.len(), 1);
}

// ── mr_merge_buffers ──────────────────────────────────────────────────────

#[test]
fn cov_mr_merge_buffers_int8() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[10,15)}").unwrap();
    let merged = mr_merge_buffers::<Int8Ops>(&a, &b).unwrap();
    let decoded = int8mr_decode(&merged).unwrap();
    assert_eq!(decoded, "{[1,5),[10,15)}");
}

#[test]
fn cov_mr_merge_buffers_overlapping() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[3,8)}").unwrap();
    let merged = mr_merge_buffers::<Int8Ops>(&a, &b).unwrap();
    let decoded = int8mr_decode(&merged).unwrap();
    assert_eq!(decoded, "{[1,8)}");
}

// ── per-type compare for date/datetime ────────────────────────────────────

#[test]
fn cov_datemr_compare() {
    let a = datemr_encode("{[2026-01-01,2026-06-30)}").unwrap();
    let b = datemr_encode("{[2026-07-01,2026-12-31)}").unwrap();
    assert_eq!(datemr_compare(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_dtmr_compare() {
    let a = dtmr_encode("{[2026-01-01 00:00:00,2026-06-30 23:59:59)}").unwrap();
    let b = dtmr_encode("{[2026-07-01 00:00:00,2026-12-31 23:59:59)}").unwrap();
    assert_eq!(dtmr_compare(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_int4mr_compare() {
    let a = int4mr_encode("{[1,5)}").unwrap();
    let b = int4mr_encode("{[10,15)}").unwrap();
    assert_eq!(int4mr_compare(&a, &b), std::cmp::Ordering::Less);
}
