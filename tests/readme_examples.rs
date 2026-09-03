// Doc-style test for README usage examples.
// Proves the SQL-shaped examples from README.md work through the actual
// registered extension surface, without needing a live server.

use villagesql::{InValue, VdfReturn};
use vsql_ranger_arranger::engine::canonical::decode;
use vsql_ranger_arranger::func::construct::{empty_for, make_for};
use vsql_ranger_arranger::func::extract::{lower_for, lower_inc_for, upper_for, upper_inc_for};
use vsql_ranger_arranger::func::predicates::{pred_binary, pred_flag};
use vsql_ranger_arranger::func::setops::{difference_for, intersect_for, merge_for, union_for};
use vsql_ranger_arranger::subtype::date::{DateOps, encode_date};
use vsql_ranger_arranger::subtype::int8::{Int8Ops, encode_int8};

fn int_ret(v: VdfReturn) -> i64 {
    match v {
        VdfReturn::Int(i) => i,
        other => panic!("expected Int, got {:?}", other),
    }
}
fn str_ret(v: VdfReturn) -> String {
    match v {
        VdfReturn::String(s) => s,
        other => panic!("expected String, got {:?}", other),
    }
}
fn bin_ret(v: VdfReturn) -> Vec<u8> {
    match v {
        VdfReturn::Binary(b) => b,
        other => panic!("expected Binary, got {:?}", other),
    }
}

// ── Scheduling / date-range examples ──────────────────────────────────────────

#[test]
fn readme_date_difference_free_slots() {
    let booked = bin_ret(make_for(encode_date, "DATERANGE")(&[
        InValue::String("2026-01-01"),
        InValue::String("2026-01-31"),
        InValue::String("[)"),
    ]));
    let hole = bin_ret(make_for(encode_date, "DATERANGE")(&[
        InValue::String("2026-01-10"),
        InValue::String("2026-01-20"),
        InValue::String("[)"),
    ]));
    let json = str_ret(difference_for::<DateOps>()(&[
        InValue::Custom(&booked),
        InValue::Custom(&hole),
    ]));
    assert_eq!(json, "[[2026-01-01,2026-01-10),[2026-01-20,2026-01-31)]");
}

#[test]
fn readme_date_intersect_no_conflict() {
    let a = bin_ret(make_for(encode_date, "DATERANGE")(&[
        InValue::String("2026-01-15"),
        InValue::String("2026-01-20"),
        InValue::String("[)"),
    ]));
    let b = bin_ret(make_for(encode_date, "DATERANGE")(&[
        InValue::String("2026-01-10"),
        InValue::String("2026-01-12"),
        InValue::String("[)"),
    ]));
    let inter = decode::<DateOps>(&bin_ret(intersect_for::<DateOps>()(&[
        InValue::Custom(&a),
        InValue::Custom(&b),
    ])))
    .unwrap();
    assert_eq!(inter, "empty");
}

// ── Anti-lossy decomposition ──────────────────────────────────────────────────

#[test]
fn readme_int8_difference_preserves_pieces() {
    let a = bin_ret(make_for(encode_int8, "INT8RANGE")(&[
        InValue::String("1"),
        InValue::String("10"),
        InValue::String("[)"),
    ]));
    let b = bin_ret(make_for(encode_int8, "INT8RANGE")(&[
        InValue::String("3"),
        InValue::String("6"),
        InValue::String("[)"),
    ]));
    let json = str_ret(difference_for::<Int8Ops>()(&[
        InValue::Custom(&a),
        InValue::Custom(&b),
    ]));
    assert_eq!(json, "[[1,3),[6,10)]");
}

// ── NULL-safe algebra ─────────────────────────────────────────────────────────

#[test]
fn readme_null_predicate_propagates() {
    let ba = encode_int8("[1,5)").unwrap();
    let args = [InValue::Custom(&ba), InValue::Null];
    assert!(matches!(
        pred_binary::<Int8Ops>(|_, _| true)(&args),
        VdfReturn::Null
    ));
}

#[test]
fn readme_null_constructor_returns_null() {
    let ret = make_for(encode_int8, "INT8RANGE")(&[
        InValue::Null,
        InValue::String("5"),
        InValue::String("[]"),
    ]);
    assert!(matches!(ret, VdfReturn::Null));
}

// ── README surface sanity checks ──────────────────────────────────────────────

#[test]
fn readme_int8_intersect_merge_surface() {
    let ba = encode_int8("[1,5)").unwrap();
    let bb = encode_int8("[3,8)").unwrap();
    assert_eq!(
        decode::<Int8Ops>(&bin_ret(intersect_for::<Int8Ops>()(&[
            InValue::Custom(&ba),
            InValue::Custom(&bb)
        ])))
        .unwrap(),
        "[3,5)"
    );

    let bc = encode_int8("[1,5)").unwrap();
    let bd = encode_int8("[10,15)").unwrap();
    assert_eq!(
        decode::<Int8Ops>(&bin_ret(merge_for::<Int8Ops>()(&[
            InValue::Custom(&bc),
            InValue::Custom(&bd)
        ])))
        .unwrap(),
        "[1,15)"
    );
}

#[test]
fn readme_int8_union_disjoint_errors() {
    let ba = encode_int8("[1,5)").unwrap();
    let bb = encode_int8("[10,15)").unwrap();
    assert!(matches!(
        union_for::<Int8Ops>()(&[InValue::Custom(&ba), InValue::Custom(&bb)]),
        VdfReturn::Error(_)
    ));
}

#[test]
fn readme_int8_accessors_and_flags() {
    let b = encode_int8("[1,5)").unwrap();
    assert_eq!(str_ret(lower_for::<Int8Ops>()(&[InValue::Custom(&b)])), "1");
    assert_eq!(str_ret(upper_for::<Int8Ops>()(&[InValue::Custom(&b)])), "5");
    assert_eq!(
        int_ret(lower_inc_for::<Int8Ops>()(&[InValue::Custom(&b)])),
        1
    );
    assert_eq!(
        int_ret(upper_inc_for::<Int8Ops>()(&[InValue::Custom(&b)])),
        0
    );

    let empty = empty_for(encode_int8)(&[]);
    assert_eq!(
        int_ret(pred_flag::<Int8Ops>(|r| r.empty)(&[InValue::Custom(
            &bin_ret(empty)
        )])),
        1
    );
}
