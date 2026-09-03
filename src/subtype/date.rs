// DATERANGE — discrete, ordinal = days since Unix epoch. AD-2: discrete -> [).
use crate::engine::canonical::{decode, encode, parse_literal};
use crate::engine::{Range, RangeSubtypeOps, persisted_length};
use chrono::NaiveDate;
use std::cmp::Ordering;

pub struct DateOps;

impl RangeSubtypeOps for DateOps {
    const ENDPOINT_BYTES: usize = 8;
    const IS_DISCRETE: bool = true;
    const TYPE_NAME: &'static str = "DATERANGE";

    fn to_ordinal(endpoint: &str) -> Result<i128, String> {
        let d = NaiveDate::parse_from_str(endpoint.trim(), "%Y-%m-%d")
            .map_err(|e| format!("DATERANGE: bad date '{endpoint}': {e}"))?;
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        Ok(d.signed_duration_since(epoch).num_days() as i128)
    }

    fn from_ordinal(ordinal: i128) -> Result<String, String> {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        epoch
            .checked_add_signed(chrono::Duration::days(ordinal as i64))
            .map(|d| d.format("%Y-%m-%d").to_string())
            .ok_or_else(|| format!("DATERANGE: ordinal {ordinal} out of representable range"))
    }
}

pub fn encode_date(s: &str) -> Result<Vec<u8>, String> {
    encode::<DateOps>(s)
}
pub fn decode_date(b: &[u8]) -> Result<String, String> {
    decode::<DateOps>(b)
}
pub fn compare_date(a: &[u8], b: &[u8]) -> Ordering {
    crate::engine::compare::range_compare(a, b)
}

#[allow(dead_code)]
pub fn parse(s: &str) -> Result<Range, String> {
    parse_literal::<DateOps>(s)
}
#[allow(dead_code)]
pub fn plen() -> usize {
    persisted_length(DateOps::ENDPOINT_BYTES)
}
