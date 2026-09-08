// Coverage slice: multirange BOUNDS and LOWER_INC.
// Exercises the new symmetry accessors for all four subtypes.

use villagesql::{InValue, VdfReturn};
use vsql_ranger_arranger::func::extract::{
    datemr_bounds, datemr_lower_inc, dtmr_bounds, dtmr_lower_inc, int4mr_bounds, int4mr_lower_inc,
    int8mr_bounds, int8mr_lower_inc,
};
use vsql_ranger_arranger::multirange_types::{
    datemr_encode, dtmr_encode, int4mr_encode, int8mr_encode,
};

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

// ── INT8 BOUNDS ───────────────────────────────────────────────────────────

#[test]
fn cov_int8mr_bounds_simple() {
    let mr = int8mr_encode("{[1,5)}").unwrap();
    assert_eq!(str_ret(int8mr_bounds(&[InValue::Custom(&mr)])), "[1,5)");
}

#[test]
fn cov_int8mr_bounds_multi() {
    let mr = int8mr_encode("{[1,5),[10,15)}").unwrap();
    assert_eq!(str_ret(int8mr_bounds(&[InValue::Custom(&mr)])), "[1,15)");
}

#[test]
fn cov_int8mr_bounds_null() {
    assert!(matches!(int8mr_bounds(&[InValue::Null]), VdfReturn::Null));
}

#[test]
fn cov_int8mr_bounds_wrong_type() {
    assert!(matches!(
        int8mr_bounds(&[InValue::Int(5)]),
        VdfReturn::Error(_)
    ));
}

// ── INT8 LOWER_INC ────────────────────────────────────────────────────────

#[test]
fn cov_int8mr_lower_inc_true() {
    let mr = int8mr_encode("{[1,5)}").unwrap();
    assert_eq!(int_ret(int8mr_lower_inc(&[InValue::Custom(&mr)])), 1);
}

#[test]
fn cov_int8mr_lower_inc_null() {
    assert!(matches!(
        int8mr_lower_inc(&[InValue::Null]),
        VdfReturn::Null
    ));
}

// ── INT4 BOUNDS ───────────────────────────────────────────────────────────

#[test]
fn cov_int4mr_bounds_simple() {
    let mr = int4mr_encode("{[1,5)}").unwrap();
    assert_eq!(str_ret(int4mr_bounds(&[InValue::Custom(&mr)])), "[1,5)");
}

#[test]
fn cov_int4mr_bounds_multi() {
    let mr = int4mr_encode("{[1,5),[10,15)}").unwrap();
    assert_eq!(str_ret(int4mr_bounds(&[InValue::Custom(&mr)])), "[1,15)");
}

#[test]
fn cov_int4mr_lower_inc_true() {
    let mr = int4mr_encode("{[1,5)}").unwrap();
    assert_eq!(int_ret(int4mr_lower_inc(&[InValue::Custom(&mr)])), 1);
}

// ── DATE BOUNDS ───────────────────────────────────────────────────────────

#[test]
fn cov_datemr_bounds_simple() {
    let mr = datemr_encode("{[2026-01-01,2026-06-30)}").unwrap();
    assert_eq!(
        str_ret(datemr_bounds(&[InValue::Custom(&mr)])),
        "[2026-01-01,2026-06-30)"
    );
}

#[test]
fn cov_datemr_bounds_multi() {
    let mr = datemr_encode("{[2026-01-01,2026-03-31),[2026-07-01,2026-12-31)}").unwrap();
    assert_eq!(
        str_ret(datemr_bounds(&[InValue::Custom(&mr)])),
        "[2026-01-01,2026-12-31)"
    );
}

#[test]
fn cov_datemr_lower_inc_true() {
    let mr = datemr_encode("{[2026-01-01,2026-06-30)}").unwrap();
    assert_eq!(int_ret(datemr_lower_inc(&[InValue::Custom(&mr)])), 1);
}

// ── DATETIME BOUNDS ───────────────────────────────────────────────────────

#[test]
fn cov_dtmr_bounds_simple() {
    let mr = dtmr_encode("{[2026-01-01 00:00:00,2026-06-30 23:59:59)}").unwrap();
    assert_eq!(
        str_ret(dtmr_bounds(&[InValue::Custom(&mr)])),
        "[2026-01-01 00:00:00.000000,2026-06-30 23:59:59.000000)"
    );
}

#[test]
fn cov_dtmr_bounds_multi() {
    let mr = dtmr_encode(
        "{[2026-01-01 00:00:00,2026-03-31 23:59:59),[2026-07-01 00:00:00,2026-12-31 23:59:59)}",
    )
    .unwrap();
    assert_eq!(
        str_ret(dtmr_bounds(&[InValue::Custom(&mr)])),
        "[2026-01-01 00:00:00.000000,2026-12-31 23:59:59.000000)"
    );
}

#[test]
fn cov_dtmr_lower_inc_true() {
    let mr = dtmr_encode("{[2026-01-01 00:00:00,2026-06-30 23:59:59)}").unwrap();
    assert_eq!(int_ret(dtmr_lower_inc(&[InValue::Custom(&mr)])), 1);
}
