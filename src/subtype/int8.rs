// INT8RANGE — discrete, i64 ordinal. AD-2: distinct custom_type!; discrete -> [).
use crate::engine::canonical::{decode, encode, parse_literal};
use crate::engine::{Range, RangeSubtypeOps, persisted_length};
use std::cmp::Ordering;

pub struct Int8Ops;

impl RangeSubtypeOps for Int8Ops {
    const ENDPOINT_BYTES: usize = 8;
    const IS_DISCRETE: bool = true;
    const TYPE_NAME: &'static str = "INT8RANGE";

    fn to_ordinal(endpoint: &str) -> Result<i128, String> {
        endpoint
            .trim()
            .parse::<i64>()
            .map(|v| v as i128)
            .map_err(|e| format!("INT8RANGE: bad endpoint '{endpoint}': {e}"))
    }

    fn from_ordinal(ordinal: i128) -> Result<String, String> {
        Ok((ordinal as i64).to_string())
    }
}

pub fn encode_int8(s: &str) -> Result<Vec<u8>, String> {
    encode::<Int8Ops>(s)
}
pub fn decode_int8(b: &[u8]) -> Result<String, String> {
    decode::<Int8Ops>(b)
}
pub fn compare_int8(a: &[u8], b: &[u8]) -> Ordering {
    crate::engine::compare::range_compare(a, b)
}

#[allow(dead_code)]
pub fn parse(s: &str) -> Result<Range, String> {
    parse_literal::<Int8Ops>(s)
}
#[allow(dead_code)]
pub fn plen() -> usize {
    persisted_length(Int8Ops::ENDPOINT_BYTES)
}
