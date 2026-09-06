// Multirange type registrations (custom_type!) + shared encode/decode/compare/hash core.
//
// Four SQL-visible types: INT8MULTIRANGE, INT4MULTIRANGE, DATEMULTIRANGE,
// DATETIMEMULTIRANGE. Each holds 0..256 normalized, non-overlapping, ordered
// components. Storage is fixed-width per type (the server limits proved this is
// the viable path in planning/multirange-probe-discovery.md):
//
//   [u16 BE count][component_0..component_count-1][zero padding to persisted_length]
//
// Per-component encoding reuses the existing single-range encode/decode
// (subtype::*::encode_int*/decode_int* → engine::canonical::range_to_bytes/to_range),
// so multirange never re-derives endpoint ordinals from scratch — it just packs and
// unpacks the already-canonical component bytes.
//
// Persisted lengths (all far under the server's 65532-byte ceiling proved by the probe):
//   INT8 / DATE / DATETIME : 2 + 256*17 = 4354
//   INT4                  : 2 + 256* 9 = 2306
//
// Hash is over the full physical buffer (count + components + zero padding). Since
// padding is deterministic zeros and count determines how many components are live,
// two values with the same logical components always produce the same buffer.
//
// Normalization lives in normalize_components below. The four registrations are thin.

use std::cmp::Ordering;

use villagesql::{TypeWithFuncs, custom_type};

use crate::engine::{
    Range, RangeSubtypeOps, canonical, compare::range_compare, flags::HEADER_LEN, persisted_length,
};
use crate::subtype;

// ── constants ──

const MAX_COMPONENTS: usize = 256;
const COUNT_BYTES: usize = 2;

const INT8MR_PERSISTED_LENGTH: usize =
    COUNT_BYTES + MAX_COMPONENTS * (HEADER_LEN + 2 * crate::subtype::int8::Int8Ops::ENDPOINT_BYTES);
const INT4MR_PERSISTED_LENGTH: usize =
    COUNT_BYTES + MAX_COMPONENTS * (HEADER_LEN + 2 * crate::subtype::int4::Int4Ops::ENDPOINT_BYTES);
const DATEMR_PERSISTED_LENGTH: usize =
    COUNT_BYTES + MAX_COMPONENTS * (HEADER_LEN + 2 * crate::subtype::date::DateOps::ENDPOINT_BYTES);
const DTMR_PERSISTED_LENGTH: usize = COUNT_BYTES
    + MAX_COMPONENTS * (HEADER_LEN + 2 * crate::subtype::datetime::DateTimeOps::ENDPOINT_BYTES);

// ── parse helpers ──

fn parse_multirange_literal<T: RangeSubtypeOps>(s: &str) -> Result<Vec<Range>, String> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("empty") || s == "{}" || s.is_empty() {
        return Ok(Vec::new());
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 2 || chars[0] != '{' || chars[chars.len() - 1] != '}' {
        return Err(format!(
            "multirange: expected '{{...}}' or 'empty', got '{s}'"
        ));
    }
    let body = &chars[1..chars.len() - 1];
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < body.len() {
        // skip whitespace and commas between components
        if body[i].is_whitespace() || body[i] == ',' {
            i += 1;
            continue;
        }
        // each component starts with '[' or '('
        if body[i] != '[' && body[i] != '(' {
            return Err(format!(
                "multirange: expected range start at position {i} in '{s}'"
            ));
        }
        let start = i;
        // find matching ']' or ')': ranges don't nest, so it's the next closing bracket
        i += 1;
        while i < body.len() && body[i] != ']' && body[i] != ')' {
            i += 1;
        }
        if i >= body.len() {
            return Err(format!("multirange: unclosed range in '{s}'"));
        }
        let lit: String = body[start..=i].iter().collect();
        let r = canonical::parse_literal::<T>(&lit)?;
        ranges.push(r);
        i += 1;
    }
    Ok(ranges)
}

// ── normalization ──

fn normalize_components<T: RangeSubtypeOps>(
    mut components: Vec<Range>,
) -> Result<Vec<Range>, String> {
    if components.is_empty() {
        return Ok(components);
    }
    // canonicalize each component first (e.g. (0,0) -> empty in discrete types)
    for r in &mut components {
        let c = crate::engine::canonicalize::<T>(r);
        *r = c;
    }
    // skip empty components (they collapse to nothing in a multirange)
    components.retain(|r| !r.empty);
    if components.is_empty() {
        return Ok(components);
    }
    // sort by lower bound (total order), breaking ties by upper
    components.sort_by(|a, b| {
        let a_lo = if a.lower_inf { i128::MIN } else { a.lower };
        let a_hi = if a.upper_inf { i128::MAX } else { a.upper };
        let b_lo = if b.lower_inf { i128::MIN } else { b.lower };
        let b_hi = if b.upper_inf { i128::MAX } else { b.upper };
        a_lo.cmp(&b_lo).then(a_hi.cmp(&b_hi))
    });
    // merge adjacent / overlapping and drop empty results
    let mut out = Vec::new();
    let mut cur = components[0];
    for &r in &components[1..] {
        if overlaps_or_adjacent(&cur, &r) {
            // merge: the union of cur and r
            cur = merge_ranges(&cur, &r);
        } else {
            out.push(cur);
            cur = r;
        }
    }
    out.push(cur);
    // drop any empty results from merging (e.g. overlapping ranges that collapse)
    Ok(out.into_iter().filter(|r| !r.empty).collect())
}

fn overlaps_or_adjacent(a: &Range, b: &Range) -> bool {
    crate::engine::overlaps(a, b) || crate::engine::adjacent(a, b)
}

fn merge_ranges(a: &Range, b: &Range) -> Range {
    // union of two overlapping/adjacent ranges — minimal enclosing interval
    let lo = if a.lower_inf || b.lower_inf {
        i128::MIN
    } else {
        a.lower.min(b.lower)
    };
    let hi = if a.upper_inf || b.upper_inf {
        i128::MAX
    } else {
        a.upper.max(b.upper)
    };
    Range {
        empty: false,
        lower_inf: a.lower_inf || b.lower_inf,
        upper_inf: a.upper_inf || b.upper_inf,
        lower_inc: true,
        upper_inc: false,
        lower: lo,
        upper: hi,
    }
}

// ── encode components into buffer ──

fn encode_components<T: RangeSubtypeOps>(
    components: &[Range],
    buf: &mut [u8],
) -> Result<(), String> {
    let cb = persisted_length(T::ENDPOINT_BYTES);
    if buf.len() < COUNT_BYTES + components.len() * cb {
        return Err(format!(
            "multirange: buffer too short ({}) for {} components of width {}",
            buf.len(),
            components.len(),
            cb
        ));
    }
    // count
    let count = components.len() as u16;
    buf[0] = (count >> 8) as u8;
    buf[1] = count as u8;
    // components
    for (i, r) in components.iter().enumerate() {
        let c = crate::engine::canonicalize::<T>(r);
        let comp_bytes = crate::engine::canonical::range_to_bytes::<T>(&c);
        let off = COUNT_BYTES + i * cb;
        buf[off..off + cb].copy_from_slice(&comp_bytes);
    }
    // remaining bytes stay zero (padding)
    Ok(())
}

// ── generic encode/decode/compare ──

pub fn mr_encode_inner<T: RangeSubtypeOps>(s: &str) -> Result<Vec<u8>, String> {
    let components = parse_multirange_literal::<T>(s)?;
    let normalized = normalize_components::<T>(components)?;
    if normalized.len() > MAX_COMPONENTS {
        return Err(format!(
            "multirange: {} components exceeds maximum of {}",
            normalized.len(),
            MAX_COMPONENTS
        ));
    }
    let plen = COUNT_BYTES + MAX_COMPONENTS * persisted_length(T::ENDPOINT_BYTES);
    let mut buf = vec![0u8; plen];
    encode_components::<T>(&normalized, &mut buf)?;
    Ok(buf)
}

pub fn mr_canonical_empty<T: RangeSubtypeOps>() -> Vec<u8> {
    let plen = COUNT_BYTES + MAX_COMPONENTS * persisted_length(T::ENDPOINT_BYTES);
    vec![0u8; plen]
}

pub fn mr_decode_inner<T: RangeSubtypeOps>(buf: &[u8]) -> Result<String, String> {
    let plen = COUNT_BYTES + MAX_COMPONENTS * persisted_length(T::ENDPOINT_BYTES);
    if buf.len() != plen {
        return Err(format!(
            "{}: corrupt stored length {} (expected {})",
            T::TYPE_NAME,
            buf.len(),
            plen
        ));
    }
    let count = ((buf[0] as usize) << 8) | (buf[1] as usize);
    if count > MAX_COMPONENTS {
        return Err(format!(
            "{}: component count {} exceeds maximum {}",
            T::TYPE_NAME,
            count,
            MAX_COMPONENTS
        ));
    }
    let cb = persisted_length(T::ENDPOINT_BYTES);
    let mut parts = Vec::with_capacity(count);
    for i in 0..count {
        let off = COUNT_BYTES + i * cb;
        let lit = canonical::decode::<T>(&buf[off..off + cb])?;
        parts.push(lit);
    }
    if count == 0 {
        Ok("{}".to_string())
    } else {
        Ok(format!("{{{}}}", parts.join(",")))
    }
}

pub fn mr_compare_inner<T: RangeSubtypeOps>(a: &[u8], b: &[u8]) -> Ordering {
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }
    let plen = COUNT_BYTES + MAX_COMPONENTS * persisted_length(T::ENDPOINT_BYTES);
    if a.len() != plen {
        return a.len().cmp(&b.len());
    }
    let count_a = ((a[0] as usize) << 8) | (a[1] as usize);
    let count_b = ((b[0] as usize) << 8) | (b[1] as usize);
    let n = count_a.min(count_b);
    let cb = persisted_length(T::ENDPOINT_BYTES);
    for i in 0..n {
        let a_off = COUNT_BYTES + i * cb;
        let b_off = COUNT_BYTES + i * cb;
        let a_end = a_off + cb;
        let ord = range_compare(&a[a_off..a_end], &b[b_off..a_end]);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    count_a.cmp(&count_b)
}

// ── per-type wrapper fns (module-level, in scope for custom_type!) ──

// INT8MULTIRANGE
pub fn int8mr_encode(s: &str) -> Result<Vec<u8>, String> {
    mr_encode_inner::<crate::subtype::int8::Int8Ops>(s)
}
pub fn int8mr_decode(b: &[u8]) -> Result<String, String> {
    mr_decode_inner::<crate::subtype::int8::Int8Ops>(b)
}
pub fn int8mr_compare(a: &[u8], b: &[u8]) -> Ordering {
    mr_compare_inner::<crate::subtype::int8::Int8Ops>(a, b)
}

// INT4MULTIRANGE
pub fn int4mr_encode(s: &str) -> Result<Vec<u8>, String> {
    mr_encode_inner::<crate::subtype::int4::Int4Ops>(s)
}
pub fn int4mr_decode(b: &[u8]) -> Result<String, String> {
    mr_decode_inner::<crate::subtype::int4::Int4Ops>(b)
}
pub fn int4mr_compare(a: &[u8], b: &[u8]) -> Ordering {
    mr_compare_inner::<crate::subtype::int4::Int4Ops>(a, b)
}

// DATEMULTIRANGE
pub fn datemr_encode(s: &str) -> Result<Vec<u8>, String> {
    mr_encode_inner::<crate::subtype::date::DateOps>(s)
}
pub fn datemr_decode(b: &[u8]) -> Result<String, String> {
    mr_decode_inner::<crate::subtype::date::DateOps>(b)
}
pub fn datemr_compare(a: &[u8], b: &[u8]) -> Ordering {
    mr_compare_inner::<crate::subtype::date::DateOps>(a, b)
}

// DATETIMEMULTIRANGE
pub fn dtmr_encode(s: &str) -> Result<Vec<u8>, String> {
    mr_encode_inner::<crate::subtype::datetime::DateTimeOps>(s)
}
pub fn dtmr_decode(b: &[u8]) -> Result<String, String> {
    mr_decode_inner::<crate::subtype::datetime::DateTimeOps>(b)
}
pub fn dtmr_compare(a: &[u8], b: &[u8]) -> Ordering {
    mr_compare_inner::<crate::subtype::datetime::DateTimeOps>(a, b)
}

// ── hash (shared — hashes full physical buffer) ──

pub fn mr_hash(b: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    b.hash(&mut h);
    h.finish()
}

// ── multirange algebra (lifted from single-range primitives) ──
//
// Each function decodes both operands, applies the lifted operation,
// then re-encodes the result.  The result is always canonicalized
// (merge adjacent/overlapping components, drop empties).

fn mr_overlaps_inner<T: RangeSubtypeOps>(a_buf: &[u8], b_buf: &[u8]) -> Result<bool, String> {
    let a_comps = mr_decode_to_vec::<T>(a_buf)?;
    let b_comps = mr_decode_to_vec::<T>(b_buf)?;
    for ra in &a_comps {
        for rb in &b_comps {
            if crate::engine::overlaps(ra, rb) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn mr_contains_range_inner<T: RangeSubtypeOps>(a_buf: &[u8], b_buf: &[u8]) -> Result<bool, String> {
    let a_comps = mr_decode_to_vec::<T>(a_buf)?;
    let b_comps = mr_decode_to_vec::<T>(b_buf)?;
    for rb in &b_comps {
        let mut found = false;
        for ra in &a_comps {
            if crate::engine::contains_range(ra, rb) {
                found = true;
                break;
            }
        }
        if !found {
            return Ok(false);
        }
    }
    Ok(true)
}

fn mr_intersect_inner<T: RangeSubtypeOps>(a_buf: &[u8], b_buf: &[u8]) -> Result<Vec<u8>, String> {
    let a_comps = mr_decode_to_vec::<T>(a_buf)?;
    let b_comps = mr_decode_to_vec::<T>(b_buf)?;
    let mut result = Vec::new();
    for ra in &a_comps {
        for rb in &b_comps {
            if crate::engine::overlaps(ra, rb) {
                result.push(crate::engine::intersect(ra, rb));
            }
        }
    }
    let normalized = normalize_components::<T>(result)?;
    mr_encode_components::<T>(&normalized)
}

fn mr_merge_inner<T: RangeSubtypeOps>(a_buf: &[u8], b_buf: &[u8]) -> Result<Vec<u8>, String> {
    let a_comps = mr_decode_to_vec::<T>(a_buf)?;
    let b_comps = mr_decode_to_vec::<T>(b_buf)?;
    let mut combined = a_comps;
    combined.extend(b_comps);
    let normalized = normalize_components::<T>(combined)?;
    mr_encode_components::<T>(&normalized)
}

fn mr_difference_inner<T: RangeSubtypeOps>(a_buf: &[u8], b_buf: &[u8]) -> Result<Vec<u8>, String> {
    let a_comps = mr_decode_to_vec::<T>(a_buf)?;
    let b_comps = mr_decode_to_vec::<T>(b_buf)?;
    let mut result = Vec::new();
    for ra in &a_comps {
        let mut remaining = vec![*ra];
        for rb in &b_comps {
            let mut next = Vec::new();
            for piece in remaining {
                next.extend(crate::engine::difference(&piece, rb));
            }
            remaining = next;
        }
        result.extend(remaining);
    }
    let normalized = normalize_components::<T>(result)?;
    mr_encode_components::<T>(&normalized)
}

pub fn mr_decode_to_vec<T: RangeSubtypeOps>(
    buf: &[u8],
) -> Result<Vec<crate::engine::Range>, String> {
    let plen = COUNT_BYTES + MAX_COMPONENTS * persisted_length(T::ENDPOINT_BYTES);
    if buf.len() != plen {
        return Err(format!(
            "{}: corrupt stored length {} (expected {})",
            T::TYPE_NAME,
            buf.len(),
            plen
        ));
    }
    let count = ((buf[0] as usize) << 8) | (buf[1] as usize);
    if count > MAX_COMPONENTS {
        return Err(format!(
            "{}: component count {} exceeds maximum {}",
            T::TYPE_NAME,
            count,
            MAX_COMPONENTS
        ));
    }
    let cb = persisted_length(T::ENDPOINT_BYTES);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let off = COUNT_BYTES + i * cb;
        let r = canonical::to_range::<T>(&buf[off..off + cb])?;
        out.push(r);
    }
    Ok(out)
}

pub fn int8mr_decode_to_vec(buf: &[u8]) -> Result<Vec<crate::engine::Range>, String> {
    mr_decode_to_vec::<subtype::int8::Int8Ops>(buf)
}
pub fn int4mr_decode_to_vec(buf: &[u8]) -> Result<Vec<crate::engine::Range>, String> {
    mr_decode_to_vec::<subtype::int4::Int4Ops>(buf)
}
pub fn datemr_decode_to_vec(buf: &[u8]) -> Result<Vec<crate::engine::Range>, String> {
    mr_decode_to_vec::<subtype::date::DateOps>(buf)
}
pub fn dtmr_decode_to_vec(buf: &[u8]) -> Result<Vec<crate::engine::Range>, String> {
    mr_decode_to_vec::<subtype::datetime::DateTimeOps>(buf)
}

pub fn mr_encode_components<T: RangeSubtypeOps>(components: &[Range]) -> Result<Vec<u8>, String> {
    if components.len() > MAX_COMPONENTS {
        return Err(format!(
            "multirange: {} components exceeds maximum of {}",
            components.len(),
            MAX_COMPONENTS
        ));
    }
    let plen = COUNT_BYTES + MAX_COMPONENTS * persisted_length(T::ENDPOINT_BYTES);
    let mut buf = vec![0u8; plen];
    let count = components.len() as u16;
    buf[0] = (count >> 8) as u8;
    buf[1] = count as u8;
    let cb = persisted_length(T::ENDPOINT_BYTES);
    for (i, r) in components.iter().enumerate() {
        let comp_bytes = range_to_bytes_canonical::<T>(r);
        let off = COUNT_BYTES + i * cb;
        buf[off..off + cb].copy_from_slice(&comp_bytes);
    }
    Ok(buf)
}

fn range_to_bytes_canonical<T: RangeSubtypeOps>(r: &Range) -> Vec<u8> {
    let c = crate::engine::canonicalize::<T>(r);
    canonical::range_to_bytes::<T>(&c)
}

// ── type registrations ──

// ── public algebra wrappers ──

pub fn int8mr_overlaps(a: &[u8], b: &[u8]) -> Result<bool, String> {
    mr_overlaps_inner::<subtype::int8::Int8Ops>(a, b)
}
pub fn int8mr_contains_range(a: &[u8], b: &[u8]) -> Result<bool, String> {
    mr_contains_range_inner::<subtype::int8::Int8Ops>(a, b)
}
pub fn int8mr_intersect(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_intersect_inner::<subtype::int8::Int8Ops>(a, b)
}
pub fn int8mr_merge(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_merge_inner::<subtype::int8::Int8Ops>(a, b)
}
pub fn int8mr_difference(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_difference_inner::<subtype::int8::Int8Ops>(a, b)
}

pub fn int4mr_overlaps(a: &[u8], b: &[u8]) -> Result<bool, String> {
    mr_overlaps_inner::<subtype::int4::Int4Ops>(a, b)
}
pub fn int4mr_contains_range(a: &[u8], b: &[u8]) -> Result<bool, String> {
    mr_contains_range_inner::<subtype::int4::Int4Ops>(a, b)
}
pub fn int4mr_intersect(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_intersect_inner::<subtype::int4::Int4Ops>(a, b)
}
pub fn int4mr_merge(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_merge_inner::<subtype::int4::Int4Ops>(a, b)
}
pub fn int4mr_difference(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_difference_inner::<subtype::int4::Int4Ops>(a, b)
}

pub fn datemr_overlaps(a: &[u8], b: &[u8]) -> Result<bool, String> {
    mr_overlaps_inner::<subtype::date::DateOps>(a, b)
}
pub fn datemr_contains_range(a: &[u8], b: &[u8]) -> Result<bool, String> {
    mr_contains_range_inner::<subtype::date::DateOps>(a, b)
}
pub fn datemr_intersect(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_intersect_inner::<subtype::date::DateOps>(a, b)
}
pub fn datemr_merge(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_merge_inner::<subtype::date::DateOps>(a, b)
}
pub fn datemr_difference(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_difference_inner::<subtype::date::DateOps>(a, b)
}

pub fn dtmr_overlaps(a: &[u8], b: &[u8]) -> Result<bool, String> {
    mr_overlaps_inner::<subtype::datetime::DateTimeOps>(a, b)
}
pub fn dtmr_contains_range(a: &[u8], b: &[u8]) -> Result<bool, String> {
    mr_contains_range_inner::<subtype::datetime::DateTimeOps>(a, b)
}
pub fn dtmr_intersect(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_intersect_inner::<subtype::datetime::DateTimeOps>(a, b)
}
pub fn dtmr_merge(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_merge_inner::<subtype::datetime::DateTimeOps>(a, b)
}
pub fn dtmr_difference(a: &[u8], b: &[u8]) -> Result<Vec<u8>, String> {
    mr_difference_inner::<subtype::datetime::DateTimeOps>(a, b)
}

// ── test helpers ──

/// Decode a multirange literal through encode then decode, returning the live components.
pub fn roundtrip_components<T: RangeSubtypeOps>(lit: &str) -> Result<Vec<Range>, String> {
    let encoded = mr_encode_inner::<T>(lit)?;
    mr_decode_to_vec::<T>(&encoded)
}

/// Lifted overlaps: true iff any component pair overlaps.
pub fn lift_overlaps<T: RangeSubtypeOps>(a: &[Range], b: &[Range]) -> bool {
    for ra in a {
        for rb in b {
            if crate::engine::overlaps(ra, rb) {
                return true;
            }
        }
    }
    false
}

/// Lifted contains_range: true iff every component in `b` is contained by some component in `a`.
pub fn lift_contains_range<T: RangeSubtypeOps>(a: &[Range], b: &[Range]) -> bool {
    for rb in b {
        let mut found = false;
        for ra in a {
            if crate::engine::contains_range(ra, rb) {
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

/// Lifted intersect: intersect every component pair, then normalize.
pub fn lift_intersect<T: RangeSubtypeOps>(a: &[Range], b: &[Range]) -> Vec<Range> {
    let mut out = Vec::new();
    for ra in a {
        for rb in b {
            if crate::engine::overlaps(ra, rb) {
                out.push(crate::engine::intersect(ra, rb));
            }
        }
    }
    normalize_components::<T>(out).unwrap_or_default()
}

/// Lifted merge: normalize the union of both component lists.
pub fn lift_merge<T: RangeSubtypeOps>(a: &[Range], b: &[Range]) -> Vec<Range> {
    let mut combined = a.to_vec();
    combined.extend(b);
    normalize_components::<T>(combined).unwrap_or_default()
}

/// Lifted difference: subtract every component of `b` from each component of `a`, then normalize.
pub fn lift_difference<T: RangeSubtypeOps>(a: &[Range], b: &[Range]) -> Vec<Range> {
    let mut out = Vec::new();
    for ra in a {
        let mut remaining = vec![*ra];
        for rb in b {
            let mut next = Vec::new();
            for piece in remaining {
                next.extend(crate::engine::difference(&piece, rb));
            }
            remaining = next;
        }
        out.extend(remaining);
    }
    normalize_components::<T>(out).unwrap_or_default()
}

// ── type registrations ──

pub fn int8mr() -> TypeWithFuncs {
    custom_type!(
        type_name: "INT8MULTIRANGE",
        persisted_length: INT8MR_PERSISTED_LENGTH as i64,
        max_decode_buffer_length: INT8MR_PERSISTED_LENGTH as i64,
        encode: int8mr_encode,
        decode: int8mr_decode,
        compare: int8mr_compare,
        hash: mr_hash,
    )
}

pub fn int4mr() -> TypeWithFuncs {
    custom_type!(
        type_name: "INT4MULTIRANGE",
        persisted_length: INT4MR_PERSISTED_LENGTH as i64,
        max_decode_buffer_length: INT4MR_PERSISTED_LENGTH as i64,
        encode: int4mr_encode,
        decode: int4mr_decode,
        compare: int4mr_compare,
        hash: mr_hash,
    )
}

pub fn datemr() -> TypeWithFuncs {
    custom_type!(
        type_name: "DATEMULTIRANGE",
        persisted_length: DATEMR_PERSISTED_LENGTH as i64,
        max_decode_buffer_length: DATEMR_PERSISTED_LENGTH as i64,
        encode: datemr_encode,
        decode: datemr_decode,
        compare: datemr_compare,
        hash: mr_hash,
    )
}

pub fn dtmr() -> TypeWithFuncs {
    custom_type!(
        type_name: "DATETIMEMULTIRANGE",
        persisted_length: DTMR_PERSISTED_LENGTH as i64,
        max_decode_buffer_length: DTMR_PERSISTED_LENGTH as i64,
        encode: dtmr_encode,
        decode: dtmr_decode,
        compare: dtmr_compare,
        hash: mr_hash,
    )
}
