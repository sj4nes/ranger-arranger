// Coverage slice: func/extract.rs.
// Exercises the dead-code per-type accessor wrappers and multirange_make/lower.

use villagesql::{InValue, VdfReturn};
use vsql_ranger_arranger::func::extract::{
    datemr_lower, datemr_make, dtmr_lower, dtmr_make, int4mr_lower, int4mr_make, int8_lower,
    int8_lower_inc, int8_upper, int8_upper_inc, int8mr_isempty, int8mr_length, int8mr_lower,
    int8mr_make, int8mr_nth, int8mr_upper, int8mr_upper_inc,
};
use vsql_ranger_arranger::subtype::int8::{Int8Ops, encode_int8};

fn str_ret(v: VdfReturn) -> String {
    match v {
        VdfReturn::String(s) => s,
        other => panic!("expected String, got {:?}", other),
    }
}

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

// ── int8 single-range accessors ────────────────────────────────────────────

#[test]
fn cov_extract_int8_lower() {
    let b = encode_int8("[1,5)").unwrap();
    assert_eq!(str_ret(int8_lower(&[InValue::Custom(&b)])), "1");
}

#[test]
fn cov_extract_int8_upper() {
    let b = encode_int8("[1,5)").unwrap();
    assert_eq!(str_ret(int8_upper(&[InValue::Custom(&b)])), "5");
}

#[test]
fn cov_extract_int8_lower_inc() {
    let b = encode_int8("[1,5)").unwrap();
    assert_eq!(int_ret(int8_lower_inc(&[InValue::Custom(&b)])), 1);
}

#[test]
fn cov_extract_int8_upper_inc() {
    let b = encode_int8("[1,5)").unwrap();
    assert_eq!(int_ret(int8_upper_inc(&[InValue::Custom(&b)])), 0);
}

#[test]
fn cov_extract_int8_lower_null() {
    assert!(matches!(int8_lower(&[InValue::Null]), VdfReturn::Null));
}

#[test]
fn cov_extract_int8_lower_wrong_type() {
    assert!(matches!(
        int8_lower(&[InValue::Int(5)]),
        VdfReturn::Error(_)
    ));
}

// ── multirange make/lower ──────────────────────────────────────────────────

#[test]
fn cov_extract_int8mr_make() {
    let ret = int8mr_make(&[InValue::String("{[1,5)}")]);
    assert!(matches!(ret, VdfReturn::Binary(_)));
}

#[test]
fn cov_extract_int8mr_make_null() {
    assert!(matches!(int8mr_make(&[InValue::Null]), VdfReturn::Null));
}

#[test]
fn cov_extract_int8mr_make_wrong_type() {
    assert!(matches!(
        int8mr_make(&[InValue::Int(5)]),
        VdfReturn::Error(_)
    ));
}

#[test]
fn cov_extract_int8mr_lower() {
    let mr = int8mr_make(&[InValue::String("{[1,5)}")]);
    let bytes = bin_ret(mr);
    // multirange_lower returns the full decoded literal
    assert_eq!(str_ret(int8mr_lower(&[InValue::Custom(&bytes)])), "{[1,5)}");
}

#[test]
fn cov_extract_int8mr_lower_null() {
    assert!(matches!(int8mr_lower(&[InValue::Null]), VdfReturn::Null));
}

#[test]
fn cov_extract_int4mr_make() {
    let ret = int4mr_make(&[InValue::String("{[1,5)}")]);
    assert!(matches!(ret, VdfReturn::Binary(_)));
}

#[test]
fn cov_extract_int4mr_lower() {
    let mr = int4mr_make(&[InValue::String("{[1,5)}")]);
    let bytes = bin_ret(mr);
    assert_eq!(str_ret(int4mr_lower(&[InValue::Custom(&bytes)])), "{[1,5)}");
}

#[test]
fn cov_extract_datemr_make() {
    let ret = datemr_make(&[InValue::String("{[2026-01-01,2026-06-30)}")]);
    assert!(matches!(ret, VdfReturn::Binary(_)));
}

#[test]
fn cov_extract_datemr_lower() {
    let mr = datemr_make(&[InValue::String("{[2026-01-01,2026-06-30)}")]);
    let bytes = bin_ret(mr);
    assert_eq!(
        str_ret(datemr_lower(&[InValue::Custom(&bytes)])),
        "{[2026-01-01,2026-06-30)}"
    );
}

#[test]
fn cov_extract_dtmr_make() {
    let ret = dtmr_make(&[InValue::String(
        "{[2026-01-01 00:00:00,2026-06-30 23:59:59)}",
    )]);
    assert!(matches!(ret, VdfReturn::Binary(_)));
}

#[test]
fn cov_extract_dtmr_lower() {
    let mr = dtmr_make(&[InValue::String(
        "{[2026-01-01 00:00:00,2026-06-30 23:59:59)}",
    )]);
    let bytes = bin_ret(mr);
    assert_eq!(
        str_ret(dtmr_lower(&[InValue::Custom(&bytes)])),
        "{[2026-01-01 00:00:00.000000,2026-06-30 23:59:59.000000)}"
    );
}

// ── multirange upper/isempty/length ────────────────────────────────────────

#[test]
fn cov_extract_int8mr_upper() {
    let mr = int8mr_make(&[InValue::String("{[1,5)}")]);
    let bytes = bin_ret(mr);
    assert_eq!(str_ret(int8mr_upper(&[InValue::Custom(&bytes)])), "5");
}

#[test]
fn cov_extract_int8mr_upper_inc() {
    let mr = int8mr_make(&[InValue::String("{[1,5)}")]);
    let bytes = bin_ret(mr);
    assert_eq!(int_ret(int8mr_upper_inc(&[InValue::Custom(&bytes)])), 0);
}

#[test]
fn cov_extract_int8mr_isempty_false() {
    let mr = int8mr_make(&[InValue::String("{[1,5)}")]);
    let bytes = bin_ret(mr);
    assert_eq!(int_ret(int8mr_isempty(&[InValue::Custom(&bytes)])), 0);
}

#[test]
fn cov_extract_int8mr_isempty_true() {
    let mr = int8mr_make(&[InValue::String("{}")]);
    let bytes = bin_ret(mr);
    assert_eq!(int_ret(int8mr_isempty(&[InValue::Custom(&bytes)])), 1);
}

#[test]
fn cov_extract_int8mr_length() {
    let mr = int8mr_make(&[InValue::String("{[1,5),[10,15)}")]);
    let bytes = bin_ret(mr);
    assert_eq!(int_ret(int8mr_length(&[InValue::Custom(&bytes)])), 2);
}

#[test]
fn cov_extract_int8mr_nth_first() {
    let mr = int8mr_make(&[InValue::String("{[1,5),[10,15)}")]);
    let bytes = bin_ret(mr);
    let ret = int8mr_nth(&[InValue::Int(0), InValue::Custom(&bytes)]);
    let nth_bytes = bin_ret(ret);
    let decoded = vsql_ranger_arranger::engine::canonical::decode::<Int8Ops>(&nth_bytes).unwrap();
    assert_eq!(decoded, "[1,5)");
}

#[test]
fn cov_extract_int8mr_nth_second() {
    let mr = int8mr_make(&[InValue::String("{[1,5),[10,15)}")]);
    let bytes = bin_ret(mr);
    let ret = int8mr_nth(&[InValue::Int(1), InValue::Custom(&bytes)]);
    let nth_bytes = bin_ret(ret);
    let decoded = vsql_ranger_arranger::engine::canonical::decode::<Int8Ops>(&nth_bytes).unwrap();
    assert_eq!(decoded, "[10,15)");
}

#[test]
fn cov_extract_int8mr_nth_oob() {
    let mr = int8mr_make(&[InValue::String("{[1,5)}")]);
    let bytes = bin_ret(mr);
    assert!(matches!(
        int8mr_nth(&[InValue::Int(5), InValue::Custom(&bytes)]),
        VdfReturn::Null
    ));
}

#[test]
fn cov_extract_int8mr_nth_null_idx() {
    let mr = int8mr_make(&[InValue::String("{[1,5)}")]);
    let bytes = bin_ret(mr);
    assert!(matches!(
        int8mr_nth(&[InValue::Null, InValue::Custom(&bytes)]),
        VdfReturn::Null
    ));
}
