// Coverage slice: subtype/int8.rs.
// Exercises parse, plen, compare_int8, and the Int8Ops ordinal conversion edge cases.

use vsql_ranger_arranger::engine::RangeSubtypeOps;
use vsql_ranger_arranger::subtype::int8::{
    Int8Ops, compare_int8, decode_int8, encode_int8, parse, plen,
};

#[test]
fn cov_int8_parse_simple() {
    let r = parse("[1,5)").unwrap();
    assert!(!r.empty);
    assert_eq!(r.lower, 1);
    assert_eq!(r.upper, 5);
    assert!(r.lower_inc);
    assert!(!r.upper_inc);
}

#[test]
fn cov_int8_parse_empty() {
    let r = parse("empty").unwrap();
    assert!(r.empty);
}

#[test]
fn cov_int8_parse_inclusive_upper() {
    let r = parse("[1,5]").unwrap();
    assert_eq!(r.lower, 1);
    assert_eq!(r.upper, 5);
    assert!(r.lower_inc);
    assert!(r.upper_inc);
}

#[test]
fn cov_int8_parse_negative() {
    let r = parse("[-10,-5)").unwrap();
    assert_eq!(r.lower, -10);
    assert_eq!(r.upper, -5);
}

#[test]
fn cov_int8_parse_infinity() {
    let r = parse("[-infinity,5)").unwrap();
    assert!(r.lower_inf);
    assert_eq!(r.upper, 5);
    assert!(r.lower_inc);
}

#[test]
fn cov_int8_parse_plus_infinity() {
    let r = parse("[1,+infinity)").unwrap();
    assert!(!r.lower_inf);
    assert!(r.upper_inf);
    assert!(!r.upper_inc);
}

#[test]
fn cov_int8_parse_invalid() {
    assert!(parse("not-a-range").is_err());
    assert!(parse("[1,2,3)").is_err());
}

#[test]
fn cov_int8_plen() {
    let p = plen();
    // INT8RANGE: 8 bytes per endpoint + 1 byte for flags = 17
    assert_eq!(p, 17);
}

#[test]
fn cov_int8_compare_equal() {
    let a = encode_int8("[1,5)").unwrap();
    let b = encode_int8("[1,5)").unwrap();
    assert_eq!(compare_int8(&a, &b), std::cmp::Ordering::Equal);
}

#[test]
fn cov_int8_compare_less() {
    let a = encode_int8("[1,5)").unwrap();
    let b = encode_int8("[3,8)").unwrap();
    assert_eq!(compare_int8(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_int8_compare_greater() {
    let a = encode_int8("[10,20)").unwrap();
    let b = encode_int8("[1,5)").unwrap();
    assert_eq!(compare_int8(&a, &b), std::cmp::Ordering::Greater);
}

#[test]
fn cov_int8_compare_empty() {
    let a = encode_int8("empty").unwrap();
    let b = encode_int8("[1,5)").unwrap();
    assert_eq!(compare_int8(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_int8_to_ordinal() {
    assert_eq!(Int8Ops::to_ordinal("42").unwrap(), 42);
    assert_eq!(Int8Ops::to_ordinal("  -10  ").unwrap(), -10);
    assert_eq!(Int8Ops::to_ordinal("0").unwrap(), 0);
}

#[test]
fn cov_int8_to_ordinal_invalid() {
    assert!(Int8Ops::to_ordinal("not-a-number").is_err());
    assert!(Int8Ops::to_ordinal("").is_err());
    assert!(Int8Ops::to_ordinal("3.14").is_err());
}

#[test]
fn cov_int8_from_ordinal() {
    assert_eq!(Int8Ops::from_ordinal(42).unwrap(), "42");
    assert_eq!(Int8Ops::from_ordinal(-10).unwrap(), "-10");
    assert_eq!(Int8Ops::from_ordinal(0).unwrap(), "0");
}

#[test]
fn cov_int8_encode_decode_roundtrip() {
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
        let encoded = encode_int8(input).unwrap();
        let decoded = decode_int8(&encoded).unwrap();
        assert_eq!(decoded, *expected, "roundtrip failed for: {}", input);
    }
}

#[test]
fn cov_int8_encode_invalid() {
    assert!(encode_int8("not-a-range").is_err());
    assert!(encode_int8("[1,2,3)").is_err());
}

#[test]
fn cov_int8_decode_invalid() {
    assert!(decode_int8(&[]).is_err());
    assert!(decode_int8(&[1, 2, 3]).is_err());
}
