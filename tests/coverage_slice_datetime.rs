// Coverage slice: subtype/datetime.rs.
// Exercises to_ordinal, from_ordinal, parse, encode/decode roundtrip, and compare.

use vsql_ranger_arranger::engine::RangeSubtypeOps;
use vsql_ranger_arranger::subtype::datetime::{
    DateTimeOps, compare_datetime, decode_datetime, encode_datetime, parse, plen,
};

#[test]
fn cov_datetime_parse_simple() {
    let r = parse("[2026-01-01 00:00:00,2026-06-30 23:59:59)").unwrap();
    assert!(!r.empty);
    assert_eq!(
        r.lower,
        DateTimeOps::to_ordinal("2026-01-01 00:00:00").unwrap()
    );
    assert_eq!(
        r.upper,
        DateTimeOps::to_ordinal("2026-06-30 23:59:59").unwrap()
    );
    assert!(r.lower_inc);
    assert!(!r.upper_inc);
}

#[test]
fn cov_datetime_parse_empty() {
    let r = parse("empty").unwrap();
    assert!(r.empty);
}

#[test]
fn cov_datetime_parse_inclusive_upper() {
    let r = parse("[2026-01-01 00:00:00,2026-06-30 23:59:59]").unwrap();
    assert_eq!(
        r.lower,
        DateTimeOps::to_ordinal("2026-01-01 00:00:00").unwrap()
    );
    assert_eq!(
        r.upper,
        DateTimeOps::to_ordinal("2026-06-30 23:59:59").unwrap()
    );
    assert!(r.lower_inc);
    assert!(r.upper_inc);
}

#[test]
fn cov_datetime_parse_infinity() {
    let r = parse("[-infinity,2026-06-30 23:59:59)").unwrap();
    assert!(r.lower_inf);
    assert_eq!(
        r.upper,
        DateTimeOps::to_ordinal("2026-06-30 23:59:59").unwrap()
    );
    assert!(r.lower_inc);
}

#[test]
fn cov_datetime_parse_plus_infinity() {
    let r = parse("[2026-01-01 00:00:00,+infinity)").unwrap();
    assert!(!r.lower_inf);
    assert!(r.upper_inf);
    assert!(!r.upper_inc);
}

#[test]
fn cov_datetime_parse_invalid() {
    assert!(parse("not-a-datetime").is_err());
    assert!(parse("[2026-13-01 00:00:00,2026-06-30 23:59:59)").is_err());
    assert!(parse("[2026-01-32 00:00:00,2026-06-30 23:59:59)").is_err());
}

#[test]
fn cov_datetime_plen() {
    let p = plen();
    // DATETIMERANGE: 8 bytes per endpoint + 1 byte for flags = 17
    assert_eq!(p, 17);
}

#[test]
fn cov_datetime_compare_equal() {
    let a = encode_datetime("[2026-01-01 00:00:00,2026-06-30 23:59:59)").unwrap();
    let b = encode_datetime("[2026-01-01 00:00:00,2026-06-30 23:59:59)").unwrap();
    assert_eq!(compare_datetime(&a, &b), std::cmp::Ordering::Equal);
}

#[test]
fn cov_datetime_compare_less() {
    let a = encode_datetime("[2026-01-01 00:00:00,2026-06-30 23:59:59)").unwrap();
    let b = encode_datetime("[2026-07-01 00:00:00,2026-12-31 23:59:59)").unwrap();
    assert_eq!(compare_datetime(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_datetime_compare_greater() {
    let a = encode_datetime("[2027-01-01 00:00:00,2027-06-30 23:59:59)").unwrap();
    let b = encode_datetime("[2026-01-01 00:00:00,2026-06-30 23:59:59)").unwrap();
    assert_eq!(compare_datetime(&a, &b), std::cmp::Ordering::Greater);
}

#[test]
fn cov_datetime_compare_empty() {
    let a = encode_datetime("empty").unwrap();
    let b = encode_datetime("[2026-01-01 00:00:00,2026-06-30 23:59:59)").unwrap();
    assert_eq!(compare_datetime(&a, &b), std::cmp::Ordering::Less);
}

#[test]
fn cov_datetime_to_ordinal() {
    // Epoch
    assert_eq!(DateTimeOps::to_ordinal("1970-01-01 00:00:00").unwrap(), 0);
    // One second after epoch
    assert_eq!(
        DateTimeOps::to_ordinal("1970-01-01 00:00:01").unwrap(),
        1_000_000
    );
    // One second before epoch
    assert_eq!(
        DateTimeOps::to_ordinal("1969-12-31 23:59:59").unwrap(),
        -1_000_000
    );
    // With microseconds
    assert_eq!(
        DateTimeOps::to_ordinal("1970-01-01 00:00:00.500000").unwrap(),
        500_000
    );
}

#[test]
fn cov_datetime_to_ordinal_invalid() {
    assert!(DateTimeOps::to_ordinal("not-a-datetime").is_err());
    assert!(DateTimeOps::to_ordinal("").is_err());
    assert!(DateTimeOps::to_ordinal("2026-13-01 00:00:00").is_err());
    assert!(DateTimeOps::to_ordinal("2026-01-32 00:00:00").is_err());
}

#[test]
fn cov_datetime_from_ordinal() {
    // Epoch
    assert_eq!(
        DateTimeOps::from_ordinal(0).unwrap(),
        "1970-01-01 00:00:00.000000"
    );
    // One second after epoch
    assert_eq!(
        DateTimeOps::from_ordinal(1_000_000).unwrap(),
        "1970-01-01 00:00:01.000000"
    );
    // One second before epoch
    assert_eq!(
        DateTimeOps::from_ordinal(-1_000_000).unwrap(),
        "1969-12-31 23:59:59.000000"
    );
    // With microseconds
    assert_eq!(
        DateTimeOps::from_ordinal(500_000).unwrap(),
        "1970-01-01 00:00:00.500000"
    );
}

#[test]
fn cov_datetime_encode_decode_roundtrip() {
    let cases = [
        (
            "[2026-01-01 00:00:00,2026-06-30 23:59:59)",
            "[2026-01-01 00:00:00.000000,2026-06-30 23:59:59.000000)",
        ),
        (
            "[2020-01-01 00:00:00,2020-12-31 23:59:59)",
            "[2020-01-01 00:00:00.000000,2020-12-31 23:59:59.000000)",
        ),
        ("empty", "empty"),
        (
            "[-infinity,2026-06-30 23:59:59)",
            "(-infinity,2026-06-30 23:59:59.000000)",
        ),
        (
            "[2026-01-01 00:00:00,+infinity)",
            "[2026-01-01 00:00:00.000000,+infinity)",
        ),
    ];
    for (input, expected) in &cases {
        let encoded = encode_datetime(input).unwrap();
        let decoded = decode_datetime(&encoded).unwrap();
        assert_eq!(decoded, *expected, "roundtrip failed for: {}", input);
    }
}

#[test]
fn cov_datetime_encode_invalid() {
    assert!(encode_datetime("not-a-datetime").is_err());
    assert!(encode_datetime("[2026-13-01 00:00:00,2026-06-30 23:59:59)").is_err());
}

#[test]
fn cov_datetime_decode_invalid() {
    assert!(decode_datetime(&[]).is_err());
    assert!(decode_datetime(&[1, 2, 3]).is_err());
}
