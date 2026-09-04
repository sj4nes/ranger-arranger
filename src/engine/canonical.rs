// AD-3 + AD-1: encode/decode are the engine's serialization of the canonical
// `Range` ordinal model into the fixed-width flag + endpoint byte layout. Called
// by each subtype's `custom_type!` `encode`/`decode` wrappers.

use crate::engine::flags::{HEADER_LEN, Header, VERSION_NIBBLE};
use crate::engine::{Range, RangeSubtypeOps, canonicalize};
use std::cmp::Ordering;

fn put_ordinal(buf: &mut [u8], off: usize, v: i128, bytes: usize) {
    // Big-endian, sign-extended fixed width. `to_be_bytes()` produces a 16-byte
    // big-endian value; take the low `bytes` bytes, which already hold the
    // sign-extended representation (high bytes are all 0x00 or 0xFF).
    let shifted = v.to_be_bytes();
    let start = 16 - bytes;
    buf[off..off + bytes].copy_from_slice(&shifted[start..16]);
}

fn get_ordinal(buf: &[u8], off: usize, bytes: usize) -> i128 {
    // Reconstruct a sign-extended i128 from `bytes` big-endian bytes: place them in
    // the LOW `bytes` of a 16-byte BE buffer, then sign-extend via the top byte.
    let mut tmp = [0u8; 16];
    let start = 16 - bytes;
    tmp[start..16].copy_from_slice(&buf[off..off + bytes]);
    // Sign-extend: if the most-significant stored byte has its top bit set, the
    // value is negative, so fill the high (unused) bytes with 0xFF.
    if buf[off] & 0x80 != 0 {
        tmp[..start].fill(0xFF);
    }
    i128::from_be_bytes(tmp)
}

/// Parse a full literal, build the `Range` model, canonicalize, serialize.
/// `txt` like `[1,5)`, `[2026-01-01,2026-02-01)`, `empty`, `[-infinity,10)`.
/// An empty/whitespace string is the empty range (the server uses it as the
/// intrinsic default for an unconstrained range column).
pub fn encode<T: RangeSubtypeOps>(txt: &str) -> Result<Vec<u8>, String> {
    let txt = if txt.trim().is_empty() { "empty" } else { txt };
    let r = parse_literal::<T>(txt)?;
    let c = canonicalize::<T>(&r);
    let bytes = T::ENDPOINT_BYTES;
    let mut buf = vec![0u8; HEADER_LEN + 2 * bytes];
    let h = Header {
        version: VERSION_NIBBLE >> 6,
        empty: c.empty,
        lower_inc: c.lower_inc,
        upper_inc: c.upper_inc,
        lower_inf: c.lower_inf,
        upper_inf: c.upper_inf,
        canonical: true,
    };
    buf[0] = h.encode();
    // Always write the ordinals (including empty / infinite bounds) so the stored form
    // round-trips byte-for-byte through decode->encode. Empty/infinite ordinals are
    // ignored by the algebra but must survive a serialize/deserialize cycle.
    put_ordinal(&mut buf, HEADER_LEN, c.lower, bytes);
    put_ordinal(&mut buf, HEADER_LEN + bytes, c.upper, bytes);
    Ok(buf)
}

/// Serialize a `Range` model directly to stored bytes, bypassing the literal
/// string round-trip used by `encode`. Used by set operations, whose result is
/// already a canonical `Range`. This avoids two chrono parse/format calls per
/// result (the dominant cost for DATE/DATETIME).
pub fn range_to_bytes<T: RangeSubtypeOps>(r: &Range) -> Vec<u8> {
    let bytes = T::ENDPOINT_BYTES;
    let mut buf = vec![0u8; HEADER_LEN + 2 * bytes];
    let h = Header {
        version: VERSION_NIBBLE >> 6,
        empty: r.empty,
        lower_inf: r.lower_inf,
        upper_inf: r.upper_inf,
        lower_inc: r.lower_inc,
        upper_inc: r.upper_inc,
        canonical: true,
    };
    buf[0] = h.encode();
    // Always write the ordinals (including empty / infinite bounds) so the stored
    // form round-trips byte-for-byte through to_range. Empty/infinite ordinals are
    // ignored by the algebra but must survive a serialize/deserialize cycle.
    put_ordinal(&mut buf, HEADER_LEN, r.lower, bytes);
    put_ordinal(&mut buf, HEADER_LEN + bytes, r.upper, bytes);
    buf
}

/// Decode stored bytes directly to the `Range` model (for predicate algebra).
pub fn to_range<T: RangeSubtypeOps>(buf: &[u8]) -> Result<Range, String> {
    let bytes = T::ENDPOINT_BYTES;
    if buf.len() != HEADER_LEN + 2 * bytes {
        return Err(format!(
            "{}: corrupt stored length {} (expected {})",
            T::TYPE_NAME,
            buf.len(),
            HEADER_LEN + 2 * bytes
        ));
    }
    let h = Header::decode(buf[0]);
    let mut r = Range::empty();
    r.empty = h.empty;
    r.lower_inf = h.lower_inf;
    r.upper_inf = h.upper_inf;
    r.lower_inc = h.lower_inc;
    r.upper_inc = h.upper_inc;
    // Always restore the ordinals (including empty / infinite bounds) so the model
    // matches what encode/range_to_bytes wrote; the algebra ignores empty/infinite
    // ordinals regardless. This keeps the stored form byte-stable across round-trips.
    r.lower = get_ordinal(buf, HEADER_LEN, bytes);
    r.upper = get_ordinal(buf, HEADER_LEN + bytes, bytes);
    Ok(r)
}

/// Decode stored bytes back to a canonical text literal.
pub fn decode<T: RangeSubtypeOps>(buf: &[u8]) -> Result<String, String> {
    let bytes = T::ENDPOINT_BYTES;
    if buf.len() != HEADER_LEN + 2 * bytes {
        return Err(format!(
            "{}: corrupt stored length {} (expected {})",
            T::TYPE_NAME,
            buf.len(),
            HEADER_LEN + 2 * bytes
        ));
    }
    let h = Header::decode(buf[0]);
    if h.empty {
        return Ok("empty".to_string());
    }
    let lower = if h.lower_inf {
        "-infinity".to_string()
    } else {
        T::from_ordinal(get_ordinal(buf, HEADER_LEN, bytes))
            .map_err(|e| format!("{}: {}", T::TYPE_NAME, e))?
    };
    let upper = if h.upper_inf {
        "+infinity".to_string()
    } else {
        T::from_ordinal(get_ordinal(buf, HEADER_LEN + bytes, bytes))
            .map_err(|e| format!("{}: {}", T::TYPE_NAME, e))?
    };
    let lb = if h.lower_inf { "(" } else { "[" };
    let rb = if h.upper_inf {
        ")"
    } else if h.upper_inc {
        "]"
    } else {
        ")"
    };
    Ok(format!("{}{},{}{}", lb, lower, upper, rb))
}

/// Parse a literal into the `Range` model (bounds preserved; NOT yet canonicalized).
pub fn parse_literal<T: RangeSubtypeOps>(txt: &str) -> Result<Range, String> {
    let t = txt.trim();
    if t.eq_ignore_ascii_case("empty") {
        return Ok(Range::empty());
    }
    let chars: Vec<char> = t.chars().collect();
    if chars.len() < 5 || (chars[0] != '[' && chars[0] != '(') {
        return Err(format!("{}: invalid range literal '{}'", T::TYPE_NAME, t));
    }
    let (lb, rb) = (chars[0], chars[chars.len() - 1]);
    if rb != ']' && rb != ')' {
        return Err(format!("{}: invalid range literal '{}'", T::TYPE_NAME, t));
    }
    let inner = &t[1..t.len() - 1];
    let comma = inner
        .rfind(',')
        .ok_or_else(|| format!("{}: missing ',' in '{}'", T::TYPE_NAME, t))?;
    let (ls, us) = inner.split_at(comma);
    let us = &us[1..];
    let lower = ls.trim();
    let upper = us.trim();

    let lower_inf = lower.eq_ignore_ascii_case("-infinity") || lower.eq_ignore_ascii_case("-inf");
    let upper_inf = upper.eq_ignore_ascii_case("+infinity")
        || upper.eq_ignore_ascii_case("+inf")
        || upper.eq_ignore_ascii_case("infinity")
        || upper.eq_ignore_ascii_case("inf");
    if lower_inf && upper_inf {
        return Err(format!(
            "{}: both endpoints cannot be infinite",
            T::TYPE_NAME
        ));
    }
    // Reject reversed endpoints (FR-4.3).
    if !lower_inf && !upper_inf {
        let lo = T::to_ordinal(lower)?;
        let hi = T::to_ordinal(upper)?;
        if lo > hi {
            return Err(format!(
                "{}: reversed endpoints ({lo} > {hi})",
                T::TYPE_NAME,
                lo = lo,
                hi = hi
            ));
        }
    }
    Ok(Range {
        empty: false,
        lower_inf,
        upper_inf,
        lower_inc: lb == '[',
        upper_inc: rb == ']',
        lower: if lower_inf { 0 } else { T::to_ordinal(lower)? },
        upper: if upper_inf { 0 } else { T::to_ordinal(upper)? },
    })
}

/// Engine-owned byte comparison for `custom_type!` `compare` (AD-1). Subtype-agnostic.
pub fn compare_bytes(a: &[u8], b: &[u8]) -> Ordering {
    if a.len() != b.len() || a.is_empty() {
        return a.len().cmp(&b.len());
    }
    let ha = Header::decode(a[0]);
    let hb = Header::decode(b[0]);
    let ra = bytes_to_range(&ha, a);
    let rb = bytes_to_range(&hb, b);
    crate::engine::total_cmp(&ra, &rb)
}

fn bytes_to_range(h: &Header, buf: &[u8]) -> Range {
    if buf.len() < HEADER_LEN {
        return Range::empty();
    }
    let bytes = (buf.len() - HEADER_LEN) / 2;
    let mut r = Range::empty();
    r.empty = h.empty;
    r.lower_inf = h.lower_inf;
    r.upper_inf = h.upper_inf;
    r.lower_inc = h.lower_inc;
    r.upper_inc = h.upper_inc;
    // Always restore the ordinals (including empty / infinite bounds) for byte-stable compare.
    r.lower = get_ordinal(buf, HEADER_LEN, bytes);
    r.upper = get_ordinal(buf, HEADER_LEN + bytes, bytes);
    r
}
