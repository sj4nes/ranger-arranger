// Coverage slice: subtype/int4.rs.
// Exercises parse, plen, compare_int4, and the Int4Ops ordinal conversion edge cases.

use vsql_ranger_arranger::engine::RangeSubtypeOps;
use vsql_ranger_arranger::subtype::int4::{
    Int4Ops, compare_int4, decode_int4, encode_int4, parse, plen,
};

#[test]
fn cov_int4_parse_simple() {
    let r = parse("[1,5)").unwrap();
    assert!(!r.empty);
    assert_eq!(r.lower, 1);
    assert_eq!(r.upper, 5);
    assert!(r.lower_inc);
    assert!(!r.upper_inc);
}

#[test]
fn cov_int4_parse_empty() {
    let r = parse("empty").unwrap();
    assert!(r.empty);
}

#[test]
fn cov_int4_parse_inclusive_upper() {
    let r = parse("[1,5]").unwrap();
    assert_eq!(r.lower, 1);
    assert_eq!(r.upper, 5);
    assert!(r.lower_inc);
    assert!(r.upper_inc);
}

#[test]
fn cov_int4_parse_negative() {
    let r = parse("[-10,-5)").unwrap();
    assert_eq!(r.lower, -10);
    assert_eq!(r.upper, -5);
}

#[test]
fn cov_int4_parse_infinity() {
    let r = parse("[-infinity,5)").unwrap();
    assert!(r.lower_inf);
    assert_eq!(r.upper, 5);
    assert!(r.lower_inc);
}

#[test]
fn cov_int4_parse_plus_infinity() {
    let r = parse("[1,+infinity)").unwrap();
    assert!(!r.lower_inf);
    assert!(r.upper_inf);
    // Canonical form: infinity is always open
    assert!(!r.upper_inc);
}

#[test]
fn cov_int4_parse_invalid() {
    assert!(parse("not-a-range").is_err());
    assert!(parse("[1,2,3)").is_err());
}

#[test]
fn cov_int4_plen() {
    let p = plen();
    // INT4RANGE: 4 bytes per endpoint + 1 byte for flags = 9
    assert_eq!(p, 9);
}

#[test]
fn cov_int4_compare_equal() {
    let a = encode_int4("[1,5)").unwrap();
    let b = encode_int4("[1,5)").unwrap();
    assert_eq!(compare_int4(&a, &b), std::cmp::Ordering::Equal);
}

#[test]
fn cov_int4_compare_less() {
    let a = encode_int4("[1,5)").unwrap();
    let b = encode_int4("[3,8)").unwrap();
    assert_eq!(compare_int4(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_int4_compare_greater() {
    let a = encode_int4("[10,20)").unwrap();
    let b = encode_int4("[1,5)").unwrap();
    assert_eq!(compare_int4(&a, &b), std::cmp::Ordering::Greater);
}

#[test]
fn cov_int4_compare_empty() {
    let a = encode_int4("empty").unwrap();
    let b = encode_int4("[1,5)").unwrap();
    assert_eq!(compare_int4(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_int4_to_ordinal() {
    assert_eq!(Int4Ops::to_ordinal("42").unwrap(), 42);
    assert_eq!(Int4Ops::to_ordinal("  -10  ").unwrap(), -10);
    assert_eq!(Int4Ops::to_ordinal("0").unwrap(), 0);
}

#[test]
fn cov_int4_to_ordinal_invalid() {
    assert!(Int4Ops::to_ordinal("not-a-number").is_err());
    assert!(Int4Ops::to_ordinal("").is_err());
    assert!(Int4Ops::to_ordinal("3.14").is_err());
}

#[test]
fn cov_int4_from_ordinal() {
    assert_eq!(Int4Ops::from_ordinal(42).unwrap(), "42");
    assert_eq!(Int4Ops::from_ordinal(-10).unwrap(), "-10");
    assert_eq!(Int4Ops::from_ordinal(0).unwrap(), "0");
}

#[test]
fn cov_int4_encode_decode_roundtrip() {
    // Note: [1,5] canonicalizes to [1,6) for discrete types
    // [-infinity,5) canonicalizes to (-infinity,5) since infinity is always open
    let cases = [
        ("[1,5)", "[1,5)"),
        ("[0,100)", "[0,100)"),
        ("[-50,-10)", "[-50,-10)"),
        ("empty", "empty"),
        ("[1,5]", "[1,6)"),
        ("[-infinity,5)", "(-infinity,5)"),
        ("[1,+infinity)", "[1,+infinity)"),
    ];
    for (input, expected) in &cases {
        let encoded = encode_int4(input).unwrap();
        let decoded = decode_int4(&encoded).unwrap();
        assert_eq!(decoded, *expected, "roundtrip failed for: {}", input);
    }
}

#[test]
fn cov_int4_encode_invalid() {
    assert!(encode_int4("not-a-range").is_err());
    assert!(encode_int4("[1,2,3)").is_err());
}

#[test]
fn cov_int4_decode_invalid() {
    assert!(decode_int4(&[]).is_err());
    assert!(decode_int4(&[1, 2, 3]).is_err());
}
