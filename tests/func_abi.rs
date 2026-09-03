// Func-layer ABI test (ABI-4): drives the exact `func!` wrapper builders that the
// `extension!` block registers, with real `InValue`/`VdfReturn` values. This exercises
// the server-facing contract (NULL-explicit, deterministic, type-routed) without a live
// VEF server: each builder is the closure the macro calls under the hood.
//
// Surface coverage: constructors (RANGE_MAKE/EMPTY), predicates (OVERLAPS/CONTAINS_RANGE/
// ADJACENT/BEFORE/EQUALS/ISEMPTY), accessors (LOWER/UPPER/LOWER_INC/UPPER_INC), and
// set ops (INTERSECT/MERGE/UNION/DIFFERENCE/LENGTH). For INT8RANGE (discrete) unless
// noted. The returned `custom` bytes are decoded back to a literal and compared, so the
// test pins both the ABI shape AND the persisted value.

use villagesql::{InValue, VdfReturn};
use vsql_ranger_arranger::engine::canonical::{decode, to_range};
use vsql_ranger_arranger::engine::{adjacent, before, contains_range, equals, overlaps};
use vsql_ranger_arranger::func::construct::{empty_for, make_for};
use vsql_ranger_arranger::func::extract::{lower_for, lower_inc_for, upper_for, upper_inc_for};
use vsql_ranger_arranger::func::predicates::{pred_binary, pred_flag};
use vsql_ranger_arranger::func::setops::{
    difference_for, intersect_for, length_for, merge_for, union_for,
};
use vsql_ranger_arranger::subtype::int8::{Int8Ops, encode_int8};

/// Encode a literal into INT8RANGE bytes (owned).
fn bytes(lit: &str) -> Vec<u8> {
    encode_int8(lit).expect("encode literal")
}

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

// ── Constructors ──────────────────────────────────────────────────────────────
#[test]
fn func_make_builds_canonical_binary() {
    let args = [
        InValue::String("1"),
        InValue::String("5"),
        InValue::String("[)"),
    ];
    let ret = make_for(encode_int8, "INT8RANGE")(&args);
    let b = bin_ret(ret);
    assert_eq!(decode::<Int8Ops>(&b).unwrap(), "[1,5)"); // already `[)` -> unchanged
}

#[test]
fn func_make_inclusive_upper_canonicalizes() {
    // upper-inclusive literal [1,5] -> stored as [1,6)
    let args = [
        InValue::String("1"),
        InValue::String("5"),
        InValue::String("[]"),
    ];
    let ret = make_for(encode_int8, "INT8RANGE")(&args);
    let b = bin_ret(ret);
    assert_eq!(decode::<Int8Ops>(&b).unwrap(), "[1,6)");
}

#[test]
fn func_empty_returns_empty_binary() {
    let ret = empty_for(encode_int8)(&[]);
    let b = bin_ret(ret);
    let r = to_range::<Int8Ops>(&b).unwrap();
    assert!(r.empty);
}

#[test]
fn func_make_null_input_yields_null() {
    let args = [InValue::Null, InValue::String("5"), InValue::String("[]")];
    assert!(matches!(
        make_for(encode_int8, "INT8RANGE")(&args),
        VdfReturn::Null
    ));
}

#[test]
fn func_make_wrong_type_yields_error() {
    let args = [InValue::Int(1), InValue::String("5"), InValue::String("[]")];
    assert!(matches!(
        make_for(encode_int8, "INT8RANGE")(&args),
        VdfReturn::Error(_)
    ));
}

// ── Predicates ────────────────────────────────────────────────────────────────
#[test]
fn func_overlaps_and_friends() {
    let ba = bytes("[1,5)");
    let bb = bytes("[3,8)");
    let args = [InValue::Custom(&ba), InValue::Custom(&bb)];
    assert_eq!(int_ret(pred_binary::<Int8Ops>(overlaps)(&args)), 1);
    assert_eq!(int_ret(pred_binary::<Int8Ops>(contains_range)(&args)), 0); // [1,5) !contains [3,8)
    assert_eq!(int_ret(pred_binary::<Int8Ops>(adjacent)(&args)), 0);
    assert_eq!(int_ret(pred_binary::<Int8Ops>(before)(&args)), 0); // overlap -> not before
    // [1,5) before [7,9) -> true (strict gap; [6,9) would be adjacent, not before)
    let bc = bytes("[7,9)");
    let args2 = [InValue::Custom(&ba), InValue::Custom(&bc)];
    assert_eq!(int_ret(pred_binary::<Int8Ops>(before)(&args2)), 1);
    // adjacency vs before: [1,5) and [5,9) touch at 5 -> not before, adjacent
    let bcc = bytes("[5,9)");
    let args3 = [InValue::Custom(&ba), InValue::Custom(&bcc)];
    assert_eq!(int_ret(pred_binary::<Int8Ops>(before)(&args3)), 0);
    assert_eq!(int_ret(pred_binary::<Int8Ops>(adjacent)(&args3)), 1);
    assert_eq!(int_ret(pred_binary::<Int8Ops>(equals)(&args3)), 0); // [1,5) != [5,9)
    // equal: same point-set [1,5] (inclusive) == [1,6) (canonical)
    let bz = bytes("[1,5]");
    let bd = bytes("[1,6)");
    let args_eq = [InValue::Custom(&bz), InValue::Custom(&bd)];
    assert_eq!(int_ret(pred_binary::<Int8Ops>(equals)(&args_eq)), 1);
}

#[test]
fn func_isempty_flag() {
    let be = bytes("empty");
    assert_eq!(
        int_ret(pred_flag::<Int8Ops>(|r| r.empty)(&[InValue::Custom(&be)])),
        1
    );
}

#[test]
fn func_predicate_null_yields_null() {
    let ba = bytes("[1,5)");
    let args = [InValue::Custom(&ba), InValue::Null];
    assert!(matches!(
        pred_binary::<Int8Ops>(overlaps)(&args),
        VdfReturn::Null
    ));
}

// ── Accessors ─────────────────────────────────────────────────────────────────
#[test]
fn func_lower_upper_strings() {
    let ba = bytes("[1,5)");
    assert_eq!(
        str_ret(lower_for::<Int8Ops>()(&[InValue::Custom(&ba)])),
        "1"
    );
    let bb = bytes("[1,5)");
    assert_eq!(
        str_ret(upper_for::<Int8Ops>()(&[InValue::Custom(&bb)])),
        "5"
    ); // canonical upper exclusive
}

#[test]
fn func_lower_upper_infinite() {
    let ba = bytes("[-inf,5)");
    assert_eq!(
        str_ret(lower_for::<Int8Ops>()(&[InValue::Custom(&ba)])),
        "-infinity"
    );
    let bb = bytes("[1,+inf)");
    assert_eq!(
        str_ret(upper_for::<Int8Ops>()(&[InValue::Custom(&bb)])),
        "+infinity"
    );
}

#[test]
fn func_bound_inclusivity_flags() {
    let ba = bytes("[1,5)");
    assert_eq!(
        int_ret(lower_inc_for::<Int8Ops>()(&[InValue::Custom(&ba)])),
        1
    );
    assert_eq!(
        int_ret(upper_inc_for::<Int8Ops>()(&[InValue::Custom(&ba)])),
        0
    ); // canonical upper exclusive
}

// ── Set operations ────────────────────────────────────────────────────────────
#[test]
fn func_intersect_merge() {
    let ba = bytes("[1,5)");
    let bb = bytes("[3,8)");
    let args = [InValue::Custom(&ba), InValue::Custom(&bb)];
    let inter = decode::<Int8Ops>(&bin_ret(intersect_for::<Int8Ops>()(&args))).unwrap();
    assert_eq!(inter, "[3,5)");
    let bc = bytes("[1,5)");
    let bd = bytes("[10,15)");
    let args2 = [InValue::Custom(&bc), InValue::Custom(&bd)];
    let merged = decode::<Int8Ops>(&bin_ret(merge_for::<Int8Ops>()(&args2))).unwrap();
    assert_eq!(merged, "[1,15)"); // enclosing interval, gap included (by spec)
}

#[test]
fn func_union_disjoint_errors() {
    let ba = bytes("[1,5)");
    let bb = bytes("[10,15)");
    let args = [InValue::Custom(&ba), InValue::Custom(&bb)];
    assert!(matches!(union_for::<Int8Ops>()(&args), VdfReturn::Error(_)));
}

#[test]
fn func_difference_json_pieces() {
    let ba = bytes("[1,10)");
    let bb = bytes("[3,6)");
    let args = [InValue::Custom(&ba), InValue::Custom(&bb)];
    let json = str_ret(difference_for::<Int8Ops>()(&args));
    assert_eq!(json, "[[1,3),[6,10)]");
}

#[test]
fn func_length_ordinal_width() {
    let ba = bytes("[1,5)");
    assert_eq!(int_ret(length_for::<Int8Ops>()(&[InValue::Custom(&ba)])), 4);
    let bb = bytes("empty");
    assert_eq!(int_ret(length_for::<Int8Ops>()(&[InValue::Custom(&bb)])), 0);
}

#[test]
fn func_setop_null_yields_null() {
    let ba = bytes("[1,5)");
    let args = [InValue::Custom(&ba), InValue::Null];
    assert!(matches!(intersect_for::<Int8Ops>()(&args), VdfReturn::Null));
}
