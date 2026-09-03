// DATETIMERANGE — continuous, ordinal = microseconds since Unix epoch.
// AD-2: preserves supplied bound inclusivity (continuous).
use crate::engine::canonical::{decode, encode, parse_literal};
use crate::engine::{Range, RangeSubtypeOps, persisted_length};
use chrono::NaiveDateTime;
use std::cmp::Ordering;

pub struct DateTimeOps;

impl RangeSubtypeOps for DateTimeOps {
    const ENDPOINT_BYTES: usize = 8;
    const IS_DISCRETE: bool = false;
    const TYPE_NAME: &'static str = "DATETIMERANGE";

    fn to_ordinal(endpoint: &str) -> Result<i128, String> {
        let e = endpoint.trim();
        let dt = NaiveDateTime::parse_from_str(e, "%Y-%m-%d %H:%M:%S%.f")
            .or_else(|_| NaiveDateTime::parse_from_str(e, "%Y-%m-%d %H:%M:%S"))
            .map_err(|err| format!("DATETIMERANGE: bad datetime '{e}': {err}"))?;
        // Microsecond precision; guard the i64->i128 widening.
        let micros = dt.and_utc().timestamp_micros();
        Ok(micros as i128)
    }

    fn from_ordinal(ordinal: i128) -> Result<String, String> {
        let secs = ordinal / 1_000_000;
        let micros = (ordinal % 1_000_000) as u32;
        chrono::DateTime::from_timestamp(secs as i64, micros * 1000)
            .map(|dt| dt.naive_utc().format("%Y-%m-%d %H:%M:%S%.6f").to_string())
            .ok_or_else(|| format!("DATETIMERANGE: ordinal {ordinal} out of representable range"))
    }
}

pub fn encode_datetime(s: &str) -> Result<Vec<u8>, String> {
    encode::<DateTimeOps>(s)
}
pub fn decode_datetime(b: &[u8]) -> Result<String, String> {
    decode::<DateTimeOps>(b)
}
pub fn compare_datetime(a: &[u8], b: &[u8]) -> Ordering {
    crate::engine::compare::range_compare(a, b)
}

#[allow(dead_code)]
pub fn parse(s: &str) -> Result<Range, String> {
    parse_literal::<DateTimeOps>(s)
}
#[allow(dead_code)]
pub fn plen() -> usize {
    persisted_length(DateTimeOps::ENDPOINT_BYTES)
}
