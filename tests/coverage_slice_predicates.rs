// Coverage slice: predicates.rs.
// Exercises the per-type contains_point wrappers that are marked #[allow(dead_code)]
// in predicates.rs, plus the multirange algebra wrappers in setops.rs.

use villagesql::{InValue, VdfReturn};
use vsql_ranger_arranger::engine::RangeSubtypeOps;
use vsql_ranger_arranger::func::predicates::{
    date_contains_point, datetime_contains_point, int4_contains_point, int8_contains_point,
};
use vsql_ranger_arranger::func::setops::{
    datemr_contains_range, datemr_difference, datemr_intersect, datemr_merge, datemr_overlaps,
    dtmr_contains_range, dtmr_difference, dtmr_intersect, dtmr_merge, dtmr_overlaps,
    int4mr_contains_range, int4mr_difference, int4mr_intersect, int4mr_merge, int4mr_overlaps,
    int8mr_contains_range, int8mr_difference, int8mr_intersect, int8mr_merge, int8mr_overlaps,
};
use vsql_ranger_arranger::multirange_types::{
    datemr_decode_to_vec, dtmr_decode_to_vec, int4mr_decode_to_vec, int8mr_decode_to_vec,
};
use vsql_ranger_arranger::subtype::date::encode_date;
use vsql_ranger_arranger::subtype::datetime::encode_datetime;
use vsql_ranger_arranger::subtype::int4::encode_int4;
use vsql_ranger_arranger::subtype::int8::encode_int8;
use vsql_ranger_arranger::subtype::{DateOps, DateTimeOps, Int4Ops, Int8Ops};

fn int_ret(v: VdfReturn) -> i64 {
    match v {
        VdfReturn::Int(i) => i,
        other => panic!("expected Int, got {:?}", other),
    }
}

fn bin_ret(v: VdfReturn) -> Vec<u8> {
    match v {
        VdfReturn::Binary(b) => b,
        other => panic!("expected Binary, got {:?}", other),
    }
}

// ── INT8 predicates ─────────────────────────────────────────────────────────

#[test]
fn cov_int8_contains_point_yes() {
    let range_bytes = encode_int8("[1,5)").unwrap();
    let args = [InValue::Custom(&range_bytes), InValue::Int(3)];
    assert_eq!(int_ret(int8_contains_point(&args)), 1);
}

#[test]
fn cov_int8_contains_point_no() {
    let range_bytes = encode_int8("[1,5)").unwrap();
    let args = [InValue::Custom(&range_bytes), InValue::Int(7)];
    assert_eq!(int_ret(int8_contains_point(&args)), 0);
}

#[test]
fn cov_int8_contains_point_empty_range() {
    let range_bytes = encode_int8("empty").unwrap();
    let args = [InValue::Custom(&range_bytes), InValue::Int(3)];
    assert_eq!(int_ret(int8_contains_point(&args)), 0);
}

#[test]
fn cov_int8_contains_point_null_point() {
    let range_bytes = encode_int8("[1,5)").unwrap();
    let args = [InValue::Custom(&range_bytes), InValue::Null];
    assert!(matches!(int8_contains_point(&args), VdfReturn::Null));
}

// ── INT4 predicates ─────────────────────────────────────────────────────────

#[test]
fn cov_int4_contains_point_yes() {
    let range_bytes = encode_int4("[1,5)").unwrap();
    let args = [InValue::Custom(&range_bytes), InValue::Int(3)];
    assert_eq!(int_ret(int4_contains_point(&args)), 1);
}

#[test]
fn cov_int4_contains_point_no() {
    let range_bytes = encode_int4("[1,5)").unwrap();
    let args = [InValue::Custom(&range_bytes), InValue::Int(7)];
    assert_eq!(int_ret(int4_contains_point(&args)), 0);
}

// ── DATE predicates ─────────────────────────────────────────────────────────

#[test]
fn cov_date_contains_point_yes() {
    let range_bytes = encode_date("[2026-01-01,2026-12-31)").unwrap();
    let point_ordinal = DateOps::to_ordinal("2026-06-15").unwrap();
    let args = [
        InValue::Custom(&range_bytes),
        InValue::Int(point_ordinal as i64),
    ];
    assert_eq!(int_ret(date_contains_point(&args)), 1);
}

#[test]
fn cov_date_contains_point_no() {
    let range_bytes = encode_date("[2026-01-01,2026-03-31)").unwrap();
    let point_ordinal = DateOps::to_ordinal("2026-06-15").unwrap();
    let args = [
        InValue::Custom(&range_bytes),
        InValue::Int(point_ordinal as i64),
    ];
    assert_eq!(int_ret(date_contains_point(&args)), 0);
}

// ── DATETIME predicates ─────────────────────────────────────────────────────

#[test]
fn cov_datetime_contains_point_yes() {
    let range_bytes = encode_datetime("[2026-01-01 00:00:00,2026-12-31 23:59:59)").unwrap();
    let point_ordinal = DateTimeOps::to_ordinal("2026-06-15 12:00:00").unwrap();
    let args = [
        InValue::Custom(&range_bytes),
        InValue::Int(point_ordinal as i64),
    ];
    assert_eq!(int_ret(datetime_contains_point(&args)), 1);
}

#[test]
fn cov_datetime_contains_point_no() {
    let range_bytes = encode_datetime("[2026-01-01 00:00:00,2026-03-31 23:59:59)").unwrap();
    let point_ordinal = DateTimeOps::to_ordinal("2026-06-15 12:00:00").unwrap();
    let args = [
        InValue::Custom(&range_bytes),
        InValue::Int(point_ordinal as i64),
    ];
    assert_eq!(int_ret(datetime_contains_point(&args)), 0);
}

// ── Multirange algebra predicates ──────────────────────────────────────────

#[test]
fn cov_int8mr_overlaps_yes() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[3,8)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(int8mr_overlaps(&args)), 1);
}

#[test]
fn cov_int8mr_overlaps_no() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[10,15)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(int8mr_overlaps(&args)), 0);
}

#[test]
fn cov_int8mr_contains_range_yes() {
    let a = int8mr_encode("{[1,10)}").unwrap();
    let b = int8mr_encode("{[3,7)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(int8mr_contains_range(&args)), 1);
}

#[test]
fn cov_int8mr_contains_range_no() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[3,10)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(int8mr_contains_range(&args)), 0);
}

#[test]
fn cov_int8mr_intersect_nonempty() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[3,8)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(int8mr_intersect(&args));
    let decoded = int8mr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 1);
}

#[test]
fn cov_int8mr_intersect_empty() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[10,15)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(int8mr_intersect(&args));
    let decoded = int8mr_decode_to_vec(&got).unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn cov_int8mr_merge_overlapping() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[3,8)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(int8mr_merge(&args));
    let decoded = int8mr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 1);
}

#[test]
fn cov_int8mr_merge_disjoint() {
    let a = int8mr_encode("{[1,5)}").unwrap();
    let b = int8mr_encode("{[10,15)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(int8mr_merge(&args));
    let decoded = int8mr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 2);
}

#[test]
fn cov_int8mr_difference_nonempty() {
    let a = int8mr_encode("{[1,10)}").unwrap();
    let b = int8mr_encode("{[3,7)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(int8mr_difference(&args));
    let decoded = int8mr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 2);
}

// ── INT4 multirange algebra ────────────────────────────────────────────────

#[test]
fn cov_int4mr_overlaps_yes() {
    let a = int4mr_encode("{[1,5)}").unwrap();
    let b = int4mr_encode("{[3,8)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(int4mr_overlaps(&args)), 1);
}

#[test]
fn cov_int4mr_contains_range_yes() {
    let a = int4mr_encode("{[1,10)}").unwrap();
    let b = int4mr_encode("{[3,7)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(int4mr_contains_range(&args)), 1);
}

#[test]
fn cov_int4mr_intersect_nonempty() {
    let a = int4mr_encode("{[1,5)}").unwrap();
    let b = int4mr_encode("{[3,8)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(int4mr_intersect(&args));
    let decoded = int4mr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 1);
}

#[test]
fn cov_int4mr_merge_overlapping() {
    let a = int4mr_encode("{[1,5)}").unwrap();
    let b = int4mr_encode("{[3,8)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(int4mr_merge(&args));
    let decoded = int4mr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 1);
}

#[test]
fn cov_int4mr_difference_nonempty() {
    let a = int4mr_encode("{[1,10)}").unwrap();
    let b = int4mr_encode("{[3,7)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(int4mr_difference(&args));
    let decoded = int4mr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 2);
}

// ── DATE multirange algebra ────────────────────────────────────────────────

#[test]
fn cov_datemr_overlaps_yes() {
    let a = datemr_encode("{[2026-01-01,2026-01-10)}").unwrap();
    let b = datemr_encode("{[2026-01-05,2026-01-15)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(datemr_overlaps(&args)), 1);
}

#[test]
fn cov_datemr_overlaps_no() {
    let a = datemr_encode("{[2026-01-01,2026-01-10)}").unwrap();
    let b = datemr_encode("{[2026-02-01,2026-02-15)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(datemr_overlaps(&args)), 0);
}

#[test]
fn cov_datemr_contains_range_yes() {
    let a = datemr_encode("{[2026-01-01,2026-01-31)}").unwrap();
    let b = datemr_encode("{[2026-01-10,2026-01-20)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(datemr_contains_range(&args)), 1);
}

#[test]
fn cov_datemr_intersect_nonempty() {
    let a = datemr_encode("{[2026-01-01,2026-01-10)}").unwrap();
    let b = datemr_encode("{[2026-01-05,2026-01-15)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(datemr_intersect(&args));
    let decoded = datemr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 1);
}

#[test]
fn cov_datemr_merge_overlapping() {
    let a = datemr_encode("{[2026-01-01,2026-01-10)}").unwrap();
    let b = datemr_encode("{[2026-01-05,2026-01-15)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(datemr_merge(&args));
    let decoded = datemr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 1);
}

#[test]
fn cov_datemr_difference_nonempty() {
    let a = datemr_encode("{[2026-01-01,2026-01-31)}").unwrap();
    let b = datemr_encode("{[2026-01-10,2026-01-20)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(datemr_difference(&args));
    let decoded = datemr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 2);
}

// ── DATETIME multirange algebra ────────────────────────────────────────────

#[test]
fn cov_dtmr_overlaps_yes() {
    let a = dtmr_encode("{[2026-01-01 00:00:00,2026-01-10 00:00:00)}").unwrap();
    let b = dtmr_encode("{[2026-01-05 00:00:00,2026-01-15 00:00:00)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(dtmr_overlaps(&args)), 1);
}

#[test]
fn cov_dtmr_overlaps_no() {
    let a = dtmr_encode("{[2026-01-01 00:00:00,2026-01-10 00:00:00)}").unwrap();
    let b = dtmr_encode("{[2026-02-01 00:00:00,2026-02-15 00:00:00)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(dtmr_overlaps(&args)), 0);
}

#[test]
fn cov_dtmr_contains_range_yes() {
    let a = dtmr_encode("{[2026-01-01 00:00:00,2026-01-31 00:00:00)}").unwrap();
    let b = dtmr_encode("{[2026-01-10 00:00:00,2026-01-20 00:00:00)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(int_ret(dtmr_contains_range(&args)), 1);
}

#[test]
fn cov_dtmr_intersect_nonempty() {
    let a = dtmr_encode("{[2026-01-01 00:00:00,2026-01-10 00:00:00)}").unwrap();
    let b = dtmr_encode("{[2026-01-05 00:00:00,2026-01-15 00:00:00)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(dtmr_intersect(&args));
    let decoded = dtmr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 1);
}

#[test]
fn cov_dtmr_merge_overlapping() {
    let a = dtmr_encode("{[2026-01-01 00:00:00,2026-01-10 00:00:00)}").unwrap();
    let b = dtmr_encode("{[2026-01-05 00:00:00,2026-01-15 00:00:00)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(dtmr_merge(&args));
    let decoded = dtmr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 1);
}

#[test]
fn cov_dtmr_difference_nonempty() {
    let a = dtmr_encode("{[2026-01-01 00:00:00,2026-01-31 00:00:00)}").unwrap();
    let b = dtmr_encode("{[2026-01-10 00:00:00,2026-01-20 00:00:00)}").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    let got = bin_ret(dtmr_difference(&args));
    let decoded = dtmr_decode_to_vec(&got).unwrap();
    assert_eq!(decoded.len(), 2);
}

// ── Helper: encode a multirange literal ────────────────────────────────────

fn int8mr_encode(lit: &str) -> Result<Vec<u8>, String> {
    use vsql_ranger_arranger::multirange_types::mr_encode_inner;
    mr_encode_inner::<Int8Ops>(lit)
}

fn int4mr_encode(lit: &str) -> Result<Vec<u8>, String> {
    use vsql_ranger_arranger::multirange_types::mr_encode_inner;
    mr_encode_inner::<Int4Ops>(lit)
}

fn datemr_encode(lit: &str) -> Result<Vec<u8>, String> {
    use vsql_ranger_arranger::multirange_types::mr_encode_inner;
    mr_encode_inner::<DateOps>(lit)
}

fn dtmr_encode(lit: &str) -> Result<Vec<u8>, String> {
    use vsql_ranger_arranger::multirange_types::mr_encode_inner;
    mr_encode_inner::<DateTimeOps>(lit)
}
