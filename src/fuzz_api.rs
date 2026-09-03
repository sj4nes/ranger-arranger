// Fuzzing harness — single source of truth for both `cargo test`
// (tests/fuzz_harness.rs, stable, runs today) and `cargo +nightly fuzz run`
// (fuzz/fuzz_targets/*.rs, libFuzzer, once cargo-fuzz is installed).
//
// Two independent fuzz oracles (neither re-calls the engine under test):
//
// 1. `fuzz_bytes` — feeds ARBITRARY BYTES to the encoder/decoder/deserializer.
//    This is the untrusted-input path: the server hands stored bytes to
//    `to_range`/`decode`, and a malformed .veb or corrupted page could feed us
//    anything. We assert (a) no panic, (b) encode->decode and encode->to_range
//    round-trips, (c) decode never emits a literal the encoder then rejects.
//
// 2. `fuzz_algebra` — feeds RANDOM RANGES to the algebra and checks every result
//    against a brute-force set-theory oracle over a bounded integer window.
//    Independent restatement of `[)` semantics (not a copy of engine code).

use crate::engine::canonical::{decode, encode, range_to_bytes, to_range};
use crate::engine::{
    Range, canonicalize, contains_point, contains_range, difference, intersect, merge, overlaps,
};
use crate::subtype::int8::Int8Ops;

/// Bounded window the algebra oracle brute-forces over. `IN_WIN` is the bound inputs
/// must stay within (so the check runs); `OUT_WIN` is wider to give `canonicalize`
/// headroom — an inclusive upper becomes `upper+1`, so an in-window input can produce
/// an engine result one past `IN_WIN`. The oracle accepts results up to `OUT_WIN`.
const IN_WIN: i128 = 1000;
const OUT_WIN: i128 = 1100;

/// Structure-aware parser: turn arbitrary bytes into a `Range`. Deterministic,
/// total (never panics). Used by `fuzz_bytes` to build ranges from the mutator's
/// raw bytes, and a building block for the algebra fuzz.
///
/// Mirrors `engine::flags::Header::decode` bit layout:
///   byte 0: empty(0) lower_inc(1) upper_inc(2) lower_inf(3) upper_inf(4) canonical(5) reserved(6,7)
pub fn parse_range(data: &[u8]) -> Range {
    let flag = |bit: usize| data.first().copied().unwrap_or(0) & (1 << bit) != 0;
    let take_i128 = |off: usize| -> i128 {
        let mut buf = [0u8; 16];
        if off < data.len() {
            let n = (data.len() - off).min(16);
            buf[..n].copy_from_slice(&data[off..off + n]);
        }
        i128::from_be_bytes(buf)
    };
    let empty = flag(0);
    let lower_inc = flag(1);
    let upper_inc = flag(2);
    let lower_inf = flag(3);
    let upper_inf = flag(4);
    let lower = take_i128(1);
    let upper = take_i128(9);
    Range {
        empty,
        lower_inf,
        upper_inf,
        lower_inc,
        upper_inc,
        lower,
        upper,
    }
}

// ---- 1. Raw-byte round-trip / panic-safety ----

/// Assert encode/decode/to_range are panic-free and internally consistent on any
/// bytes, for both the literal-input path and the direct struct path.
pub fn fuzz_bytes(data: &[u8]) {
    // (a) Literal path: build a literal from bytes, encode it, decode it back.
    let r = parse_range(data);
    let lit = literal_from_range(&r);
    let stored = match encode::<Int8Ops>(&lit) {
        Ok(b) => b,
        Err(_) => return, // invalid literal is a legitimate rejection, not a crash
    };
    // decode must never panic and must round-trip back to a re-encodable literal.
    let back = decode::<Int8Ops>(&stored).expect("decode of our own encode must succeed");
    let re = encode::<Int8Ops>(&back).expect("re-encode of decode must succeed");
    assert_eq!(
        stored, re,
        "decode->encode not idempotent for literal {lit:?}"
    );

    // (b) Direct struct path (the server's stored-column path): encode the range
    // straight to bytes and read it back. The model MUST round-trip byte-for-byte
    // (range_to_bytes preserves infinite-bound ordinals; to_range restores them).
    // This is the stability contract the engine relies on for persisted values.
    // Clamp ordinals to Int8Ops' representable domain (i64; ENDPOINT_BYTES=8) — the
    // fixed-width format cannot hold i128 values outside i64, so out-of-domain
    // inputs are not a round-trip contract the engine promises.
    let mut r = parse_range(data);
    let lo = i64::MIN as i128;
    let hi = i64::MAX as i128;
    r.lower = r.lower.clamp(lo, hi);
    r.upper = r.upper.clamp(lo, hi);
    let stored2 = range_to_bytes::<Int8Ops>(&r);
    let rt = to_range::<Int8Ops>(&stored2).expect("to_range of our own bytes must succeed");
    assert_eq!(
        rt, r,
        "range_to_bytes -> to_range not byte-stable for {r:?}"
    );
}

// ---- 2. Differential algebra oracle ----

/// Independent point-membership: straightforward restatement of `[)` semantics with
/// infinity awareness. Deliberately NOT a call into engine code.
fn in_set(r: &Range, p: i128) -> bool {
    if r.empty {
        return false;
    }
    let lo = if r.lower_inf {
        i128::MIN
    } else if r.lower_inc {
        r.lower
    } else {
        r.lower + 1
    };
    let hi = if r.upper_inf {
        i128::MAX
    } else if r.upper_inc {
        r.upper
    } else {
        r.upper - 1
    };
    p >= lo && p <= hi
}

/// Brute-force the bounded window and return the set of contained ordinals. For
/// infinite bounds we skip the full check (covered by the literal/round-trip fuzz
/// and the proptest ordinal reference) — this oracle is sound+complete only for
/// finite ranges, which is what we restrict `fuzz_algebra` to.
fn window_set(r: &Range) -> Option<Vec<i128>> {
    if r.empty || r.lower_inf || r.upper_inf {
        return None;
    }
    if r.lower < -IN_WIN || r.upper > IN_WIN {
        return None; // out of brute-force window
    }
    let lo = if r.lower_inc { r.lower } else { r.lower + 1 };
    let hi = if r.upper_inc { r.upper } else { r.upper - 1 };
    if lo > hi {
        return Some(vec![]);
    }
    Some((lo..=hi).collect())
}

/// Differential check of the full algebra against the brute-force oracle. `a`/`b`
/// may be arbitrary; we only run the sound check when both are finite and in-window.
pub fn fuzz_algebra(a: &Range, b: &Range) {
    let (sa, sb) = match (window_set(a), window_set(b)) {
        (Some(sa), Some(sb)) => (sa, sb),
        _ => return, // infinity / out-of-window: skip (handled elsewhere)
    };
    let set_of = |s: &[i128]| -> std::collections::BTreeSet<i128> { s.iter().copied().collect() };
    let set_a = set_of(&sa);
    let set_b = set_of(&sb);

    // canonical `[)` before comparing (engine precondition: funcs feed canonical).
    let (ac, bc) = (canonicalize::<Int8Ops>(a), canonicalize::<Int8Ops>(b));

    // intersect == set intersection, as a range.
    let inter = intersect(&ac, &bc);
    let inter_set: std::collections::BTreeSet<i128> = set_a.intersection(&set_b).copied().collect();
    assert_eq!(
        range_to_window_set(&inter),
        Some(inter_set.clone()),
        "intersect mismatch"
    );

    // union / merge == minimal ENCLOSING interval over the union's span (gap filled).
    // `merge` is defined as the minimal enclosing range, NOT the set-theoretic union;
    // for disjoint inputs it spans the gap (FR-8.3). So the expected set is every
    // point from the smallest contained ordinal to the largest, inclusive.
    let mg = merge(&ac, &bc);
    let spanned_set: std::collections::BTreeSet<i128> = {
        let all: std::collections::BTreeSet<i128> = set_a.union(&set_b).copied().collect();
        if all.is_empty() {
            all
        } else {
            let mn = *all.iter().next().unwrap();
            let mx = *all.iter().next_back().unwrap();
            (mn..=mx).collect()
        }
    };
    assert_eq!(
        range_to_window_set(&mg),
        Some(spanned_set.clone()),
        "merge mismatch"
    );

    // difference == set difference, 0/1/2 pieces.
    let diff = difference(&ac, &bc);
    let diff_set: std::collections::BTreeSet<i128> = set_a.difference(&set_b).copied().collect();
    let diff_pieces: std::collections::BTreeSet<i128> = diff
        .iter()
        .flat_map(|p| range_to_window_set(p).unwrap_or_default())
        .collect();
    assert_eq!(diff_pieces, diff_set, "difference mismatch");

    // overlaps / adjacent / contains: ordinal checks against independent reference.
    let ref_overlaps = !set_a.is_disjoint(&set_b);
    assert_eq!(overlaps(&ac, &bc), ref_overlaps, "overlaps mismatch");

    // contains_range: a contains b iff b's set ⊆ a's set.
    assert_eq!(
        contains_range(&ac, &bc),
        set_b.is_subset(&set_a),
        "contains_range mismatch"
    );

    // contains_point on every window point (independent reference).
    for p in -IN_WIN..=IN_WIN {
        assert_eq!(
            contains_point(&ac, p),
            in_set(&ac, p),
            "contains_point({p}) mismatch"
        );
    }
}

/// Expand a finite range to its window set (mirror of `window_set` but for an
/// already-canonical engine result, which may be empty or single-piece).
fn range_to_window_set(r: &Range) -> Option<std::collections::BTreeSet<i128>> {
    if r.empty || r.lower_inf || r.upper_inf {
        return if r.empty {
            Some(std::collections::BTreeSet::new())
        } else {
            None
        };
    }
    if r.lower < -OUT_WIN || r.upper > OUT_WIN {
        return None;
    }
    let lo = if r.lower_inc { r.lower } else { r.lower + 1 };
    let hi = if r.upper_inc { r.upper } else { r.upper - 1 };
    if lo > hi {
        return Some(std::collections::BTreeSet::new());
    }
    Some((lo..=hi).collect())
}

// ---- helpers ----

/// Build a range literal string from a `Range` (the literal-input path).
fn literal_from_range(r: &Range) -> String {
    if r.empty {
        return "empty".to_string();
    }
    let lb = if r.lower_inc { '[' } else { '(' };
    let rb = if r.upper_inc { ']' } else { ')' };
    let lo = if r.lower_inf {
        "-infinity".to_string()
    } else {
        r.lower.to_string()
    };
    let hi = if r.upper_inf {
        "+infinity".to_string()
    } else {
        r.upper.to_string()
    };
    format!("{}{},{}{}", lb, lo, hi, rb)
}
