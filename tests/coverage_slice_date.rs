// Coverage slice: subtype/date.rs.
// Exercises to_ordinal, from_ordinal, parse, encode/decode roundtrip, and compare.

use vsql_ranger_arranger::engine::RangeSubtypeOps;
use vsql_ranger_arranger::subtype::date::{
    DateOps, compare_date, decode_date, encode_date, parse, plen,
};

#[test]
fn cov_date_parse_simple() {
    let r = parse("[2026-01-01,2026-06-30)").unwrap();
    assert!(!r.empty);
    assert_eq!(r.lower, DateOps::to_ordinal("2026-01-01").unwrap());
    assert_eq!(r.upper, DateOps::to_ordinal("2026-06-30").unwrap());
    assert!(r.lower_inc);
    assert!(!r.upper_inc);
}

#[test]
fn cov_date_parse_empty() {
    let r = parse("empty").unwrap();
    assert!(r.empty);
}

#[test]
fn cov_date_parse_inclusive_upper() {
    let r = parse("[2026-01-01,2026-06-30]").unwrap();
    assert_eq!(r.lower, DateOps::to_ordinal("2026-01-01").unwrap());
    assert_eq!(r.upper, DateOps::to_ordinal("2026-06-30").unwrap());
    assert!(r.lower_inc);
    assert!(r.upper_inc);
}

#[test]
fn cov_date_parse_negative_year() {
    // Dates before 1970 have negative ordinals
    let r = parse("[1960-01-01,1960-12-31)").unwrap();
    assert!(r.lower < 0);
    assert!(r.upper < 0);
}

#[test]
fn cov_date_parse_infinity() {
    let r = parse("[-infinity,2026-06-30)").unwrap();
    assert!(r.lower_inf);
    assert_eq!(r.upper, DateOps::to_ordinal("2026-06-30").unwrap());
    assert!(r.lower_inc);
}

#[test]
fn cov_date_parse_plus_infinity() {
    let r = parse("[2026-01-01,+infinity)").unwrap();
    assert!(!r.lower_inf);
    assert!(r.upper_inf);
    assert!(!r.upper_inc);
}

#[test]
fn cov_date_parse_invalid() {
    assert!(parse("not-a-date").is_err());
    assert!(parse("[2026-13-01,2026-06-30)").is_err());
    assert!(parse("[2026-01-32,2026-06-30)").is_err());
}

#[test]
fn cov_date_plen() {
    let p = plen();
    // DATERANGE: 8 bytes per endpoint + 1 byte for flags = 17
    assert_eq!(p, 17);
}

#[test]
fn cov_date_compare_equal() {
    let a = encode_date("[2026-01-01,2026-06-30)").unwrap();
    let b = encode_date("[2026-01-01,2026-06-30)").unwrap();
    assert_eq!(compare_date(&a, &b), std::cmp::Ordering::Equal);
}

#[test]
fn cov_date_compare_less() {
    let a = encode_date("[2026-01-01,2026-06-30)").unwrap();
    let b = encode_date("[2026-07-01,2026-12-31)").unwrap();
    assert_eq!(compare_date(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_date_compare_greater() {
    let a = encode_date("[2027-01-01,2027-06-30)").unwrap();
    let b = encode_date("[2026-01-01,2026-06-30)").unwrap();
    assert_eq!(compare_date(&a, &b), std::cmp::Ordering::Greater);
}

#[test]
fn cov_date_compare_empty() {
    let a = encode_date("empty").unwrap();
    let b = encode_date("[2026-01-01,2026-06-30)").unwrap();
    assert_eq!(compare_date(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_date_to_ordinal() {
    // Epoch day
    assert_eq!(DateOps::to_ordinal("1970-01-01").unwrap(), 0);
    // One day after epoch
    assert_eq!(DateOps::to_ordinal("1970-01-02").unwrap(), 1);
    // One day before epoch
    assert_eq!(DateOps::to_ordinal("1969-12-31").unwrap(), -1);
    // Modern date
    assert_eq!(DateOps::to_ordinal("2026-06-15").unwrap(), 20619);
}

#[test]
fn cov_date_to_ordinal_invalid() {
    assert!(DateOps::to_ordinal("not-a-date").is_err());
    assert!(DateOps::to_ordinal("").is_err());
    assert!(DateOps::to_ordinal("2026-13-01").is_err());
    assert!(DateOps::to_ordinal("2026-01-32").is_err());
}

#[test]
fn cov_date_from_ordinal() {
    // Epoch day
    assert_eq!(DateOps::from_ordinal(0).unwrap(), "1970-01-01");
    // One day after epoch
    assert_eq!(DateOps::from_ordinal(1).unwrap(), "1970-01-02");
    // One day before epoch
    assert_eq!(DateOps::from_ordinal(-1).unwrap(), "1969-12-31");
    // Modern date
    assert_eq!(DateOps::from_ordinal(20619).unwrap(), "2026-06-15");
}

#[test]
fn cov_date_from_ordinal_large() {
    // Large but valid ordinal
    assert_eq!(DateOps::from_ordinal(365).unwrap(), "1971-01-01");
    assert_eq!(DateOps::from_ordinal(-365).unwrap(), "1969-01-01");
}

#[test]
fn cov_date_encode_decode_roundtrip() {
    let cases = [
        ("[2026-01-01,2026-06-30)", "[2026-01-01,2026-06-30)"),
        ("[2020-01-01,2020-12-31)", "[2020-01-01,2020-12-31)"),
        ("empty", "empty"),
        ("[-infinity,2026-06-30)", "(-infinity,2026-06-30)"),
        ("[2026-01-01,+infinity)", "[2026-01-01,+infinity)"),
    ];
    for (input, expected) in &cases {
        let encoded = encode_date(input).unwrap();
        let decoded = decode_date(&encoded).unwrap();
        assert_eq!(decoded, *expected, "roundtrip failed for: {}", input);
    }
}

#[test]
fn cov_date_encode_invalid() {
    assert!(encode_date("not-a-date").is_err());
    assert!(encode_date("[2026-13-01,2026-06-30)").is_err());
}

#[test]
fn cov_date_decode_invalid() {
    assert!(decode_date(&[]).is_err());
    assert!(decode_date(&[1, 2, 3]).is_err());
}
