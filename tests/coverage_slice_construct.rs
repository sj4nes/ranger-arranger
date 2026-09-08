// Coverage slice: func/construct.rs.
// Exercises the int8_make and int8_empty wrappers that are #[allow(dead_code)].

use villagesql::{InValue, VdfReturn};
use vsql_ranger_arranger::func::construct::{int8_empty, int8_make};

#[test]
fn cov_construct_int8_make_basic() {
    let args = [
        InValue::String("1"),
        InValue::String("5"),
        InValue::String("[)"),
    ];
    let ret = int8_make(&args);
    match ret {
        VdfReturn::Binary(b) => {
            assert!(!b.is_empty());
        }
        other => panic!("expected Binary, got {:?}", other),
    }
}

#[test]
fn cov_construct_int8_make_inclusive() {
    let args = [
        InValue::String("1"),
        InValue::String("5"),
        InValue::String("[]"),
    ];
    let ret = int8_make(&args);
    match ret {
        VdfReturn::Binary(b) => {
            assert!(!b.is_empty());
        }
        other => panic!("expected Binary, got {:?}", other),
    }
}

#[test]
fn cov_construct_int8_make_null_first() {
    let args = [InValue::Null, InValue::String("5"), InValue::String("[)")];
    assert!(matches!(int8_make(&args), VdfReturn::Null));
}

#[test]
fn cov_construct_int8_make_null_second() {
    let args = [InValue::String("1"), InValue::Null, InValue::String("[)")];
    assert!(matches!(int8_make(&args), VdfReturn::Null));
}

#[test]
fn cov_construct_int8_make_null_third() {
    // bounds arg being null is not caught by the null guard (only lo/hi are checked),
    // so it falls through to the error case
    let args = [InValue::String("1"), InValue::String("5"), InValue::Null];
    assert!(matches!(int8_make(&args), VdfReturn::Error(_)));
}

#[test]
fn cov_construct_int8_make_wrong_type() {
    let args = [InValue::Int(1), InValue::String("5"), InValue::String("[)")];
    assert!(matches!(int8_make(&args), VdfReturn::Error(_)));
}

#[test]
fn cov_construct_int8_make_invalid_literal() {
    let args = [
        InValue::String("abc"),
        InValue::String("def"),
        InValue::String("[)"),
    ];
    assert!(matches!(int8_make(&args), VdfReturn::Error(_)));
}

#[test]
fn cov_construct_int8_empty() {
    let ret = int8_empty(&[]);
    match ret {
        VdfReturn::Binary(b) => {
            assert!(!b.is_empty());
        }
        other => panic!("expected Binary, got {:?}", other),
    }
}
