// ── Draft: parameterized multirange type, one element family (DATEMULTIRANGE) ──
//
// Purpose: show the full surface of a parameterized multirange type so the
// capacity/default design can be inspected before committing to it.
//
// Model:
//   - One `parameterized_type!` registration per SQL multirange family.
//   - Capacity (max normalized components) is the DDL parameter; default 256.
//   - On-disk value = capacity * ELEMENT_BYTES bytes, laid out as k equal slots.
//   - Each slot is the EXACT byte encoding of one element range (via the existing
//     element `range_to_bytes::<T>`), so empty slots are the canonical empty-range
//     encoding and are unambiguous (a real range that would encode identically is
//     itself canonicalized to empty before storage).
//   - Bare construction (function return / SELECT output) uses DEFAULT capacity.
//     Column-inserted text literals use the column's capacity via the from_string
//     VDF's `MaybeParams` path.
//   - Algebra/predicate VDFs operate within one capacity (cross-capacity values are
//     different SQL types under the SDK's parameterized-type model, so they never
//     meet in a call).
//
// TODOs / open questions (resolved after live server probe):
//   - MAX_COMPONENTS: pick so that max_persisted_length is accepted by the server.
//   - Whether bare MAKE results can be INSERTed into non-default columns (likely no;
//     document as a known limitation, or add an explicit capacity arg to MAKE later).
//   - Confirm `params_to_strings` / `int_to_params` wire format the server expects.

use std::cmp::Ordering;

// ── Element type operations (reused from existing subtype machinery) ──

// Reuse the existing `DateOps` from `src/subtype/date.rs`:
//   const ENDPOINT_BYTES: usize = 8;
//   const IS_DISCRETE: bool = true;
//   const TYPE_NAME: &'static str = "DATERANGE";
// plus `to_ordinal`, `from_ordinal`, `parse_literal`, `canonicalize`,
// `range_to_bytes`, `to_range`, `decode` (text), `compare_bytes`, etc.

// For this draft I assume these are in scope (imported from the existing
// `crate::subtype::date` and `crate::engine::*` modules):
use crate::engine::{
    canonicalize, compare_bytes, merge, normalize_components, overlaps, adjacent,
    parse_literal, range_to_bytes, Range, RangeSubtypeOps, to_range,
};
use crate::subtype::date::DateOps;
use villagesql::{InValue, MaybeParams, Params, Resolved, VdfReturn};

// ── Capacity constants (tentative — verify max against live server) ──

const MIN_COMPONENTS: usize = 1;
const DEFAULT_COMPONENTS: usize = 256;
// TODO: probe the server; 4096 -> max_persisted_length 69632 is larger than the
// tvector example's 32768. If the server rejects it, lower MAX and document.
const MAX_COMPONENTS: usize = 4096;

// One element slot is the exact persisted encoding of one element range.
const ELEMENT_BYTES: usize = DateOps::ENDPOINT_BYTES * 2 + 1; // 17 for date

// ── Per-column parameter struct ──

#[derive(Clone, Debug)]
pub struct DateMultirangeParams {
    pub components: usize, // column capacity: number of element slots
}

// ── Parameterized-type callbacks ──

/// Bare-integer DDL shorthand: `DATEMULTIRANGE(256)` -> `"components=256"`.
/// A value equal to DEFAULT is still written explicitly so the inferred params
/// are unambiguous; the server does not special-case the default.
pub fn datemultirange_int_to_params(n: i64) -> Result<String, String> {
    let n = n
        .try_into()
        .map_err(|_| "datemultirange: component count must be positive")?;
    if n < MIN_COMPONENTS as i64 {
        return Err(format!("datemultirange: component count {n} below minimum {MIN_COMPONENTS}"));
    }
    if n > MAX_COMPONENTS as i64 {
        return Err(format!("datemultirange: component count {n} above maximum {MAX_COMPONENTS}"));
    }
    Ok(format!("components={n}"))
}

pub fn datemultirange_parse(params: Params) -> DateMultirangeParams {
    let comp = params
        .get("components")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_COMPONENTS);
    DateMultirangeParams {
        components: comp.clamp(MIN_COMPONENTS, MAX_COMPONENTS),
    }
}

pub fn datemultirange_to_strings(p: &DateMultirangeParams) -> Vec<(String, String)> {
    vec![("components".to_string(), p.components.to_string())]
}

/// Validate the declaration and say how many bytes a value takes.
pub fn datemultirange_resolve_params(params: Params) -> Result<Resolved, String> {
    let p = datemultirange_parse(params);
    let persisted_length = p.components * ELEMENT_BYTES;
    let max_text_len = p.components * 32; // generous upper bound for the text form
    Ok(Resolved::new(persisted_length, max_text_len))
}

/// Intrinsic default for an unconstrained column: the canonical empty multirange
/// padded to DEFAULT capacity. (Computed per parameter set so a column declared
/// with a non-default capacity gets an appropriately-sized empty value.)
pub fn datemultirange_intrinsic_default(p: &DateMultirangeParams) -> Result<String, String> {
    let empty_slots = vec![Range::empty(); p.components];
    let bytes = slots_to_bytes(&empty_slots);
    // Produce the canonical text for an all-empty multirange: `{}`.
    let _ = bytes; // bytes are the stored form; text form is always `{}` for all-empty
    Ok("{}".to_string())
}

// ── Multirange text parser (shared by from_string and MAKE) ──

/// Parse a multirange text form into normalized components.
/// Accepts `{}`, `{empty}`, `{[..],[..)}`, with whitespace tolerance.
pub fn parse_multirange_text(text: &str) -> Result<Vec<Range>, String> {
    let t = text.trim();
    if t.is_empty() || t.eq_ignore_ascii_case("empty") || t == "{}" {
        return Ok(vec![]);
    }
    if !t.starts_with('{') || !t.ends_with('}') {
        return Err(format!("datemultirange: expected `{{...}}`, got '{t}'"));
    }
    let inner = &t[1..t.len() - 1];
    if inner.trim().is_empty() {
        return Ok(vec![]);
    }
    let components = split_top_level_commas(inner);
    let mut ranges = Vec::with_capacity(components.len());
    for piece in components {
        let piece = piece.trim();
        if piece.eq_ignore_ascii_case("empty") {
            continue;
        }
        let r = parse_literal::<DateOps>(piece)?;
        ranges.push(r);
    }
    let normalized = normalize_components(ranges);
    Ok(normalized)
}

/// Split `inner` on commas that occur at brace depth 0, so `[1,2),[3,4)` gives
/// `[1,2)` and `[3,4)` while the comma inside each range literal is preserved.
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    for (i, ch) in s.char_indices() {
        match ch {
            '[' | '(' => depth += 1,
            ']' | ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

// ── Slot array <-> bytes ──

/// Encode a full slot array (capacity slots, each ELEMENT_BYTES bytes) into the
/// persisted value bytes.
fn slots_to_bytes(slots: &[Range]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(slots.len() * ELEMENT_BYTES);
    for r in slots {
        buf.extend(range_to_bytes::<DateOps>(r));
    }
    buf
}

/// Decode `bytes` (exactly capacity * ELEMENT_BYTES) back into the real components,
/// skipping empty padding slots.
fn bytes_to_components(bytes: &[u8], capacity: usize) -> Result<Vec<Range>, String> {
    let expected = capacity * ELEMENT_BYTES;
    if bytes.len() != expected {
        return Err(format!(
            "datemultirange: stored length {} != expected {expected} (capacity {capacity})",
            bytes.len()
        ));
    }
    let mut comps = Vec::new();
    for i in 0..capacity {
        let slot = &bytes[i * ELEMENT_BYTES..(i + 1) * ELEMENT_BYTES];
        let r = to_range::<DateOps>(slot)?;
        if !r.empty {
            comps.push(r);
        }
    }
    Ok(comps)
}

// ── from_string VDF (the `custom_type!` `encode` for parameterized types) ──

pub fn datemultirange_encode(s: &str, maybe: &mut MaybeParams<DateMultirangeParams>) -> Result<Vec<u8>, String> {
    let components = parse_multirange_text(s)?;
    let capacity = if let Some(p) = maybe.get() {
        p.components
    } else {
        // Bare call (no column context, e.g. a function return): use the default
        // capacity and infer it so the result is self-describing.
        DEFAULT_COMPONENTS
    };
    if components.len() > capacity {
        return Err(format!(
            "datemultirange: value has {n} normalized components but column capacity is {capacity}",
            n = components.len()
        ));
    }
    let mut slots = Vec::with_capacity(capacity);
    for c in components {
        slots.push(c);
    }
    for _ in components.len()..capacity {
        slots.push(Range::empty());
    }
    // Infer default params for bare calls.
    if !maybe.is_known() {
        maybe.set(DateMultirangeParams {
            components: DEFAULT_COMPONENTS,
        });
    }
    Ok(slots_to_bytes(&slots))
}

// ── to_string VDF (the `custom_type!` `decode` for parameterized types) ──

pub fn datemultirange_decode(bytes: &[u8], p: &DateMultirangeParams) -> Result<String, String> {
    let components = bytes_to_components(bytes, p.components)?;
    if components.is_empty() {
        return Ok("{}".to_string());
    }
    let texts: Vec<String> = components
        .iter()
        .map(|r| {
            // Reuse the element's text decoder.
            crate::subtype::date::decode(r).expect("element round-trip must not fail on canonical slots")
        })
        .collect();
    Ok(format!("{{{}}}", texts.join(",")))
}

// ── compare VDF ──

pub fn datemultirange_compare(a: &[u8], b: &[u8], _p: &DateMultirangeParams) -> Ordering {
    // Both values are from the same column (same capacity), so compare the raw
    // slot arrays lexicographically. Equivalent normalized values produce identical
    // slot arrays (same padding), so they compare equal.
    a.cmp(b)
}

// ── hash VDF ──

pub fn datemultirange_hash(bytes: &[u8], _p: &DateMultirangeParams) -> usize {
    use std::hash::{Hash, Hasher};
    let mut h = crate::engine::DefaultHasher::new(); // or aHash/Id hasher per project convention
    bytes.hash(&mut h);
    h.finish() as usize
}

// ── parameterized_type! registration ──

pub fn datemultirange_type() -> villagesql::TypeWithFuncs {
    villagesql::parameterized_type!(
        type_name: "datemultirange",
        max_persisted_length: MAX_COMPONENTS * ELEMENT_BYTES,
        max_decode_buffer_length: MAX_COMPONENTS * 32,
        encode: datemultirange_encode,
        decode: datemultirange_decode,
        compare: datemultirange_compare,
        hash: datemultirange_hash,
        int_to_params: datemultirange_int_to_params,
        resolve_params: datemultirange_resolve_params,
        params_type: DateMultirangeParams,
        params_parse: datemultirange_parse,
        params_to_strings: datemultirange_to_strings,
        intrinsic_default_fn: datemultirange_intrinsic_default,
    )
}

// ── Construction / accessor / algebra VDFs (sketches) ──

/// `datemultirange_make(text)` — bare construction at DEFAULT capacity.
/// Returns a self-contained multirange value; usable in SELECT output and in
/// INSERT into DEFAULT-capacity columns. For non-default columns, insert a text
/// literal instead (the column's from_string picks up the column capacity).
pub fn datemultirange_make_impl(args: &[InValue]) -> VdfReturn {
    if args.iter().any(|v| matches!(v, InValue::Null)) {
        return VdfReturn::null();
    }
    let text = match &args[0] {
        InValue::String(s) => s.clone(),
        InValue::Null => return VdfReturn::null(),
        _ => return VdfReturn::error("datemultirange_make: expected text argument"),
    };
    let components = parse_multirange_text(&text)?;
    let capacity = DEFAULT_COMPONENTS;
    if components.len() > capacity {
        return VdfReturn::error(format!(
            "datemultirange_make: value has {n} normalized components but default capacity is {capacity}",
            n = components.len()
        ));
    }
    let mut slots = Vec::with_capacity(capacity);
    for c in components {
        slots.push(c);
    }
    for _ in components.len()..capacity {
        slots.push(Range::empty());
    }
    let bytes = slots_to_bytes(&slots);
    // VdfReturn::Binary expects the bytes; the result buffer must be large enough.
    VdfReturn::Binary(bytes)
}

/// `datemultirange_from_range(range)` — promote one element range to a one-component
/// multirange at DEFAULT capacity.
pub fn datemultirange_from_range_impl(args: &[InValue]) -> VdfReturn {
    if args.iter().any(|v| matches!(v, InValue::Null)) {
        return VdfReturn::null();
    }
    let r = match &args[0] {
        InValue::Custom(b) => to_range::<DateOps>(b)?,
        InValue::Null => return VdfReturn::null(),
        _ => return VdfReturn::error("datemultirange_from_range: expected a daterange value"),
    };
    let c = canonicalize::<DateOps>(&r);
    let capacity = DEFAULT_COMPONENTS;
    let mut slots = Vec::with_capacity(capacity);
    slots.push(c);
    for _ in 1..capacity {
        slots.push(Range::empty());
    }
    let bytes = slots_to_bytes(&slots);
    VdfReturn::Binary(bytes)
}

// ── Normalization helper (element-level) ──

/// Sort by lower bound, then merge overlapping AND adjacent components.
/// For discrete element types (date, int8, int4) adjacency merges; for continuous
/// element types, only genuine overlap merges (adjacency does not merge).
pub fn normalize_components(components: Vec<Range>) -> Vec<Range> {
    if components.is_empty() {
        return vec![];
    }
    let mut comps: Vec<Range> = components
        .into_iter()
        .filter(|r| !r.empty)
        .collect();
    if comps.is_empty() {
        return vec![];
    }
    comps.sort_by(|a, b| a.lower.cmp(&b.lower).then_with(|| a.upper.cmp(&b.upper)));
    let mut out = Vec::new();
    let mut cur = comps[0];
    for next in comps.into_iter().skip(1) {
        if overlaps(&cur, &next) || adjacent(&cur, &next) {
            cur = merge(&cur, &next);
        } else {
            out.push(cur);
            cur = next;
        }
    }
    out.push(cur);
    out
}

// NOTE: `normalize_components` above is the element-level normalization used when
// constructing from text. For the continuous element case (DATETIMEMULTIRANGE), the
// same function works because `adjacent` already consults the stored inclusivity flags
// and only returns true when the touching point is actually in the set. Discrete vs
// continuous is therefore handled correctly by the existing `adjacent`/`merge` on the
// canonical element `Range`, without a separate code path.
