// Coverage slice: predicates.rs (remaining).
// Covers the four *_contains_element wrappers and error paths in pred_binary/pred_flag/contains_point.

use villagesql::{InValue, VdfReturn};
use vsql_ranger_arranger::func::predicates::{
    date_contains_element, date_contains_point, datetime_contains_element, datetime_contains_point,
    int4_contains_element, int4_contains_point, int8_contains_element, int8_contains_point,
};
use vsql_ranger_arranger::multirange_types::{
    datemr_encode, dtmr_encode, int4mr_encode, int8mr_encode,
};
use vsql_ranger_arranger::subtype::date::encode_date;
use vsql_ranger_arranger::subtype::datetime::encode_datetime;
use vsql_ranger_arranger::subtype::int4::encode_int4;
use vsql_ranger_arranger::subtype::int8::encode_int8;

fn int_ret(v: VdfReturn) -> i64 {
    match v {
        VdfReturn::Int(i) => i,
        other => panic!("expected Int, got {:?}", other),
    }
}

// ── int8_contains_element ─────────────────────────────────────────────────
// Note: args[0] = single range (element), args[1] = multirange (container)

#[test]
fn cov_int8_contains_element_inside() {
    let elem = encode_int8("[3,7)").unwrap();
    let mr = int8mr_encode("{[1,10)}").unwrap();
    let args = [InValue::Custom(&elem), InValue::Custom(&mr)];
    assert_eq!(int_ret(int8_contains_element(&args)), 1);
}

#[test]
fn cov_int8_contains_element_outside() {
    let elem = encode_int8("[10,15)").unwrap();
    let mr = int8mr_encode("{[1,5)}").unwrap();
    let args = [InValue::Custom(&elem), InValue::Custom(&mr)];
    assert_eq!(int_ret(int8_contains_element(&args)), 0);
}

#[test]
fn cov_int8_contains_element_null_element() {
    let mr = int8mr_encode("{[1,5)}").unwrap();
    let args = [InValue::Null, InValue::Custom(&mr)];
    assert!(matches!(int8_contains_element(&args), VdfReturn::Null));
}

#[test]
fn cov_int8_contains_element_null_range() {
    let elem = encode_int8("[3,7)").unwrap();
    let args = [InValue::Custom(&elem), InValue::Null];
    assert!(matches!(int8_contains_element(&args), VdfReturn::Null));
}

#[test]
fn cov_int8_contains_element_wrong_type() {
    let mr = int8mr_encode("{[1,5)}").unwrap();
    let args = [InValue::Int(5), InValue::Custom(&mr)];
    assert!(matches!(int8_contains_element(&args), VdfReturn::Null));
}

// ── int4_contains_element ─────────────────────────────────────────────────

#[test]
fn cov_int4_contains_element_inside() {
    let elem = encode_int4("[3,7)").unwrap();
    let mr = int4mr_encode("{[1,10)}").unwrap();
    let args = [InValue::Custom(&elem), InValue::Custom(&mr)];
    assert_eq!(int_ret(int4_contains_element(&args)), 1);
}

#[test]
fn cov_int4_contains_element_outside() {
    let elem = encode_int4("[10,15)").unwrap();
    let mr = int4mr_encode("{[1,5)}").unwrap();
    let args = [InValue::Custom(&elem), InValue::Custom(&mr)];
    assert_eq!(int_ret(int4_contains_element(&args)), 0);
}

#[test]
fn cov_int4_contains_element_null_element() {
    let mr = int4mr_encode("{[1,5)}").unwrap();
    let args = [InValue::Null, InValue::Custom(&mr)];
    assert!(matches!(int4_contains_element(&args), VdfReturn::Null));
}

// ── date_contains_element ─────────────────────────────────────────────────

#[test]
fn cov_date_contains_element_inside() {
    let elem = encode_date("[2026-03-01,2026-06-30)").unwrap();
    let mr = datemr_encode("{[2026-01-01,2026-12-31)}").unwrap();
    let args = [InValue::Custom(&elem), InValue::Custom(&mr)];
    assert_eq!(int_ret(date_contains_element(&args)), 1);
}

#[test]
fn cov_date_contains_element_outside() {
    let elem = encode_date("[2026-06-01,2026-08-31)").unwrap();
    let mr = datemr_encode("{[2026-01-01,2026-03-31)}").unwrap();
    let args = [InValue::Custom(&elem), InValue::Custom(&mr)];
    assert_eq!(int_ret(date_contains_element(&args)), 0);
}

#[test]
fn cov_date_contains_element_null_element() {
    let mr = datemr_encode("{[2026-01-01,2026-12-31)}").unwrap();
    let args = [InValue::Null, InValue::Custom(&mr)];
    assert!(matches!(date_contains_element(&args), VdfReturn::Null));
}

// ── datetime_contains_element ─────────────────────────────────────────────

#[test]
fn cov_datetime_contains_element_inside() {
    let elem = encode_datetime("[2026-03-01 00:00:00,2026-06-30 23:59:59)").unwrap();
    let mr = dtmr_encode("{[2026-01-01 00:00:00,2026-12-31 23:59:59)}").unwrap();
    let args = [InValue::Custom(&elem), InValue::Custom(&mr)];
    assert_eq!(int_ret(datetime_contains_element(&args)), 1);
}

#[test]
fn cov_datetime_contains_element_outside() {
    let elem = encode_datetime("[2026-06-01 00:00:00,2026-08-31 23:59:59)").unwrap();
    let mr = dtmr_encode("{[2026-01-01 00:00:00,2026-03-31 23:59:59)}").unwrap();
    let args = [InValue::Custom(&elem), InValue::Custom(&mr)];
    assert_eq!(int_ret(datetime_contains_element(&args)), 0);
}

#[test]
fn cov_datetime_contains_element_null_element() {
    let mr = dtmr_encode("{[2026-01-01 00:00:00,2026-12-31 23:59:59)}").unwrap();
    let args = [InValue::Null, InValue::Custom(&mr)];
    assert!(matches!(datetime_contains_element(&args), VdfReturn::Null));
}

// ── contains_point error paths ────────────────────────────────────────────

#[test]
fn cov_int8_contains_point_null_range() {
    let args = [InValue::Null, InValue::Int(3)];
    assert!(matches!(int8_contains_point(&args), VdfReturn::Null));
}

#[test]
fn cov_int8_contains_point_wrong_type() {
    let args = [InValue::Int(5), InValue::Int(3)];
    assert!(matches!(int8_contains_point(&args), VdfReturn::Null));
}

#[test]
fn cov_int4_contains_point_null_range() {
    let args = [InValue::Null, InValue::Int(3)];
    assert!(matches!(int4_contains_point(&args), VdfReturn::Null));
}

#[test]
fn cov_int4_contains_point_wrong_type() {
    let args = [InValue::Int(5), InValue::Int(3)];
    assert!(matches!(int4_contains_point(&args), VdfReturn::Null));
}

#[test]
fn cov_date_contains_point_null_range() {
    let args = [InValue::Null, InValue::Int(100)];
    assert!(matches!(date_contains_point(&args), VdfReturn::Null));
}

#[test]
fn cov_date_contains_point_wrong_type() {
    let args = [InValue::Int(5), InValue::Int(100)];
    assert!(matches!(date_contains_point(&args), VdfReturn::Null));
}

#[test]
fn cov_datetime_contains_point_null_range() {
    let args = [InValue::Null, InValue::Int(100)];
    assert!(matches!(datetime_contains_point(&args), VdfReturn::Null));
}

#[test]
fn cov_datetime_contains_point_wrong_type() {
    let args = [InValue::Int(5), InValue::Int(100)];
    assert!(matches!(datetime_contains_point(&args), VdfReturn::Null));
}

// ── pred_binary error paths ───────────────────────────────────────────────

#[test]
fn cov_pred_binary_first_arg_error() {
    // First arg is invalid custom bytes -> to_range error
    let invalid = vec![0xFF; 100];
    let valid = encode_int8("[1,5)").unwrap();
    let args = [InValue::Custom(&invalid), InValue::Custom(&valid)];
    assert!(matches!(
        vsql_ranger_arranger::func::predicates::pred_binary::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(|_, _| true)(&args),
        VdfReturn::Error(_)
    ));
}

#[test]
fn cov_pred_binary_second_arg_error() {
    // Second arg is invalid custom bytes -> to_range error
    let valid = encode_int8("[1,5)").unwrap();
    let invalid = vec![0xFF; 100];
    let args = [InValue::Custom(&valid), InValue::Custom(&invalid)];
    assert!(matches!(
        vsql_ranger_arranger::func::predicates::pred_binary::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(|_, _| true)(&args),
        VdfReturn::Error(_)
    ));
}

#[test]
fn cov_pred_binary_catch_all_error() {
    // Neither arg is Custom -> catch-all error
    let args = [InValue::Int(1), InValue::Int(2)];
    assert!(matches!(
        vsql_ranger_arranger::func::predicates::pred_binary::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(|_, _| true)(&args),
        VdfReturn::Error(_)
    ));
}

// ── pred_flag paths ────────────────────────────────────────────────────────

#[test]
fn cov_pred_flag_false() {
    let b = encode_int8("[1,5)").unwrap();
    // Flag returns false for non-empty range
    assert_eq!(
        int_ret(vsql_ranger_arranger::func::predicates::pred_flag::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(|r| r.empty)(&[InValue::Custom(&b)])),
        0
    );
}

#[test]
fn cov_pred_flag_error() {
    // Invalid bytes -> to_range error
    let invalid = vec![0xFF; 100];
    assert!(matches!(
        vsql_ranger_arranger::func::predicates::pred_flag::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(|r| r.empty)(&[InValue::Custom(&invalid)]),
        VdfReturn::Error(_)
    ));
}

#[test]
fn cov_pred_flag_null() {
    assert!(matches!(
        vsql_ranger_arranger::func::predicates::pred_flag::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(|r| r.empty)(&[InValue::Null]),
        VdfReturn::Null
    ));
}

#[test]
fn cov_pred_flag_catch_all() {
    assert!(matches!(
        vsql_ranger_arranger::func::predicates::pred_flag::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(|r| r.empty)(&[InValue::Int(5)]),
        VdfReturn::Error(_)
    ));
}

// ── pred_binary: op returns false ─────────────────────────────────────────

#[test]
fn cov_pred_binary_op_false() {
    // Use a predicate that always returns false
    let a = encode_int8("[1,5)").unwrap();
    let b = encode_int8("[10,15)").unwrap();
    let args = [InValue::Custom(&a), InValue::Custom(&b)];
    assert_eq!(
        int_ret(vsql_ranger_arranger::func::predicates::pred_binary::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(|_, _| false)(&args)),
        0
    );
}

// ── pred_flag: flag returns false ─────────────────────────────────────────

#[test]
fn cov_pred_flag_false_branch() {
    // Use a flag that returns false
    let b = encode_int8("[1,5)").unwrap();
    assert_eq!(
        int_ret(vsql_ranger_arranger::func::predicates::pred_flag::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(|_| false)(&[InValue::Custom(&b)])),
        0
    );
}

// ── int8_contains_point: Err branch ──────────────────────────────────────

#[test]
fn cov_int8_contains_point_err() {
    // Invalid bytes -> to_range error
    let invalid = vec![0xFF; 100];
    assert!(matches!(
        int8_contains_point(&[InValue::Custom(&invalid), InValue::Int(3)]),
        VdfReturn::Null
    ));
}

// ── int8_contains_element: Err branch ─────────────────────────────────────

#[test]
fn cov_int8_contains_element_err() {
    // Invalid multirange bytes -> mr_contains_element error
    let invalid = vec![0xFF; 100];
    let elem = encode_int8("[3,7)").unwrap();
    assert!(matches!(
        int8_contains_element(&[InValue::Custom(&invalid), InValue::Custom(&elem)]),
        VdfReturn::Null
    ));
}

// ── int4_contains_point: Err branch ──────────────────────────────────────

#[test]
fn cov_int4_contains_point_err() {
    let invalid = vec![0xFF; 100];
    assert!(matches!(
        int4_contains_point(&[InValue::Custom(&invalid), InValue::Int(3)]),
        VdfReturn::Null
    ));
}

// ── int4_contains_element: Err branch ─────────────────────────────────────

#[test]
fn cov_int4_contains_element_err() {
    let invalid = vec![0xFF; 100];
    let elem = encode_int4("[3,7)").unwrap();
    assert!(matches!(
        int4_contains_element(&[InValue::Custom(&invalid), InValue::Custom(&elem)]),
        VdfReturn::Null
    ));
}

// ── date_contains_point: error paths ──────────────────────────────────────

#[test]
fn cov_date_contains_point_err() {
    let invalid = vec![0xFF; 100];
    assert!(matches!(
        date_contains_point(&[InValue::Custom(&invalid), InValue::Int(100)]),
        VdfReturn::Null
    ));
}

// ── date_contains_element: Err branch ─────────────────────────────────────

#[test]
fn cov_date_contains_element_err() {
    let invalid = vec![0xFF; 100];
    let elem = encode_date("[2026-03-01,2026-06-30)").unwrap();
    assert!(matches!(
        date_contains_element(&[InValue::Custom(&invalid), InValue::Custom(&elem)]),
        VdfReturn::Null
    ));
}

// ── datetime_contains_point: error paths ──────────────────────────────────

#[test]
fn cov_datetime_contains_point_err() {
    let invalid = vec![0xFF; 100];
    assert!(matches!(
        datetime_contains_point(&[InValue::Custom(&invalid), InValue::Int(100)]),
        VdfReturn::Null
    ));
}

// ── datetime_contains_element: Err branch ─────────────────────────────────

#[test]
fn cov_datetime_contains_element_err() {
    let invalid = vec![0xFF; 100];
    let elem = encode_datetime("[2026-03-01 00:00:00,2026-06-30 23:59:59)").unwrap();
    assert!(matches!(
        datetime_contains_element(&[InValue::Custom(&invalid), InValue::Custom(&elem)]),
        VdfReturn::Null
    ));
}

// ── contains_point: first arg catch-all (wrong type) ──────────────────────

#[test]
fn cov_int4_contains_point_wrong_first_arg() {
    // First arg is Int (not Custom, not Null) -> catch-all
    let args = [InValue::Int(5), InValue::Int(3)];
    assert!(matches!(int4_contains_point(&args), VdfReturn::Null));
}

// ── contains_element: first arg catch-all (wrong type) ────────────────────

#[test]
fn cov_int4_contains_element_wrong_first_arg() {
    let elem = encode_int4("[3,7)").unwrap();
    let args = [InValue::Int(5), InValue::Custom(&elem)];
    assert!(matches!(int4_contains_element(&args), VdfReturn::Null));
}

#[test]
fn cov_date_contains_point_wrong_first_arg() {
    let args = [InValue::Int(5), InValue::Int(100)];
    assert!(matches!(date_contains_point(&args), VdfReturn::Null));
}

#[test]
fn cov_date_contains_element_wrong_first_arg() {
    let elem = encode_date("[2026-03-01,2026-06-30)").unwrap();
    let args = [InValue::Int(5), InValue::Custom(&elem)];
    assert!(matches!(date_contains_element(&args), VdfReturn::Null));
}

#[test]
fn cov_datetime_contains_point_wrong_first_arg() {
    let args = [InValue::Int(5), InValue::Int(100)];
    assert!(matches!(datetime_contains_point(&args), VdfReturn::Null));
}

#[test]
fn cov_datetime_contains_element_wrong_first_arg() {
    let elem = encode_datetime("[2026-03-01 00:00:00,2026-06-30 23:59:59)").unwrap();
    let args = [InValue::Int(5), InValue::Custom(&elem)];
    assert!(matches!(datetime_contains_element(&args), VdfReturn::Null));
}
