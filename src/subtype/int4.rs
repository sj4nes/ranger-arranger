// INT4RANGE — discrete, i32 ordinal. AD-2: distinct custom_type!.
use crate::engine::canonical::{decode, encode, parse_literal};
use crate::engine::{Range, RangeSubtypeOps, persisted_length};
use std::cmp::Ordering;

pub struct Int4Ops;

impl RangeSubtypeOps for Int4Ops {
    const ENDPOINT_BYTES: usize = 4;
    const IS_DISCRETE: bool = true;
    const TYPE_NAME: &'static str = "INT4RANGE";

    fn to_ordinal(endpoint: &str) -> Result<i128, String> {
        let v: i32 = endpoint
            .trim()
            .parse()
            .map_err(|e| format!("INT4RANGE: bad endpoint '{endpoint}': {e}"))?;
        Ok(v as i128)
    }

    fn from_ordinal(ordinal: i128) -> Result<String, String> {
        Ok((ordinal as i32).to_string())
    }
}

pub fn encode_int4(s: &str) -> Result<Vec<u8>, String> {
    encode::<Int4Ops>(s)
}
pub fn decode_int4(b: &[u8]) -> Result<String, String> {
    decode::<Int4Ops>(b)
}
pub fn compare_int4(a: &[u8], b: &[u8]) -> Ordering {
    crate::engine::compare::range_compare(a, b)
}

#[allow(dead_code)]
pub fn parse(s: &str) -> Result<Range, String> {
    parse_literal::<Int4Ops>(s)
}
#[allow(dead_code)]
pub fn plen() -> usize {
    persisted_length(Int4Ops::ENDPOINT_BYTES)
}
