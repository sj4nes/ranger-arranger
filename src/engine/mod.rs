// AD-1: generic RangeEngine. The engine owns the algebra and the canonicalization
// policy + comparison contract. A subtype only supplies ordinal mapping and
// discreteness via `RangeSubtypeOps`; it never supplies its own `compare` or
// canonicalization branch (that would be a divergence the engine cannot detect).

pub mod canonical;
pub mod compare;
pub mod flags;

use crate::engine::flags::HEADER_LEN;
use std::cmp::Ordering;

/// Internal representation: every endpoint is a canonical `i128` ordinal supplied
/// by the subtype. Compare/order are uniform over ordinals, so `range_compare`
/// (compare.rs) is subtype-agnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub empty: bool,
    pub lower_inf: bool,
    pub upper_inf: bool,
    pub lower_inc: bool,
    pub upper_inc: bool,
    pub lower: i128,
    pub upper: i128,
}

impl Range {
    pub fn empty() -> Self {
        Range {
            empty: true,
            lower_inf: false,
            upper_inf: false,
            lower_inc: true, // canonical empty is `[)` form, matching canonicalize()
            upper_inc: false,
            lower: 0,
            upper: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }
}

/// Subtype contract. The engine calls these; subtypes implement them.
pub trait RangeSubtypeOps: Send + Sync + 'static {
    /// Bytes per endpoint in the on-disk format (fixed width).
    const ENDPOINT_BYTES: usize;
    /// Discrete subtypes (INT/BIGINT/DATE) canonicalize to `[)`; continuous keep bounds.
    const IS_DISCRETE: bool;
    /// Parse an endpoint literal ("5", "2026-01-01", "2026-01-01 00:00:00") to ordinal.
    fn to_ordinal(endpoint: &str) -> Result<i128, String>;
    /// Format an ordinal back to an endpoint literal.
    fn from_ordinal(ordinal: i128) -> Result<String, String>;
    /// SQL type name (e.g. "INT8RANGE").
    const TYPE_NAME: &'static str;
}

/// Persisted length = 1 header byte + 2 * endpoint bytes.
pub fn persisted_length(endpoint_bytes: usize) -> usize {
    HEADER_LEN + 2 * endpoint_bytes
}

/// Canonicalize per AD-1 single-ownership: the engine decides the policy.
/// Discrete -> `[)`: lower endpoint inclusive stays, an exclusive-open lower
/// becomes (successor, inclusive); an inclusive-closed upper becomes
/// (successor, exclusive). Continuous: bounds preserved as given (AD-2).
/// Even when one side is infinite, the finite side is normalized.
pub fn canonicalize<T: RangeSubtypeOps>(r: &Range) -> Range {
    if r.empty {
        return Range::empty();
    }
    if !T::IS_DISCRETE {
        return *r; // continuous: preserve supplied inclusivity
    }
    // Discrete -> canonical `[)`. For infinite bounds the ordinal is meaningless to
    // the algebra, but it must still ROUND-TRIP (encode -> to_range -> encode), so we
    // preserve the original ordinal rather than letting canonicalization zero/shift it.
    let lower = if r.lower_inf || r.lower_inc {
        r.lower
    } else {
        r.lower + 1 // exclusive-open lower -> next ordinal
    };
    let upper = if r.upper_inf {
        r.upper
    } else if r.upper_inc {
        r.upper + 1 // inclusive-closed upper -> exclusive successor
    } else {
        r.upper
    };
    let is_empty = !r.lower_inf && !r.upper_inf && lower >= upper;
    Range {
        // In canonical `[)` form, `[x,x)` is empty (no point satisfies x <= p < x).
        // When the result is empty, zero the ordinals so every empty range has one
        // canonical byte form (matches Range::empty()); the engine compares empties
        // by the `empty` flag, never by stale ordinals.
        empty: is_empty,
        lower_inf: r.lower_inf,
        upper_inf: r.upper_inf,
        lower_inc: true,
        upper_inc: false,
        lower: if is_empty { 0 } else { lower },
        upper: if is_empty { 0 } else { upper },
    }
}

// ---- Algebra (operates on already-canonical ranges) ----

pub fn contains_point(r: &Range, p: i128) -> bool {
    if r.empty {
        return false;
    }
    let lo_ok = if r.lower_inf { true } else { p >= r.lower };
    // Canonical `[)`: the stored `upper` is exclusive.
    let hi_ok = if r.upper_inf { true } else { p < r.upper };
    lo_ok && hi_ok
}

pub fn contains_range(a: &Range, b: &Range) -> bool {
    if b.empty {
        return true; // empty is contained in everything
    }
    if a.empty {
        return false;
    }
    let lo_ok = if a.lower_inf {
        true
    } else if b.lower_inf {
        false
    } else {
        b.lower >= a.lower
    };
    let hi_ok = if a.upper_inf {
        true
    } else if b.upper_inf {
        false
    } else {
        b.upper <= a.upper
    };
    lo_ok && hi_ok
}

/// Strict overlap: intersection nonempty.
pub fn overlaps(a: &Range, b: &Range) -> bool {
    if a.empty || b.empty {
        return false;
    }
    let a_lo = if a.lower_inf { i128::MIN } else { a.lower };
    let a_hi = if a.upper_inf { i128::MAX } else { a.upper };
    let b_lo = if b.lower_inf { i128::MIN } else { b.lower };
    let b_hi = if b.upper_inf { i128::MAX } else { b.upper };
    // Strict `[)` overlap: a shared boundary point is NOT an overlap
    // (e.g. [1,5) and [5,10) touch at 5 but do not overlap).
    a_lo < b_hi && b_lo < a_hi
}

/// Meet but do not overlap.
pub fn adjacent(a: &Range, b: &Range) -> bool {
    if a.empty || b.empty || overlaps(a, b) {
        return false;
    }
    let a_hi = if a.upper_inf { i128::MAX } else { a.upper };
    let b_hi = if b.upper_inf { i128::MAX } else { b.upper };
    let a_lo = if a.lower_inf { i128::MIN } else { a.lower };
    let b_lo = if b.lower_inf { i128::MIN } else { b.lower };
    // Canonical `[)` form: contiguous iff the exclusive upper of one equals the
    // inclusive lower of the other (no gap, no overlap).
    a_hi == b_lo || b_hi == a_lo
}

pub fn before(a: &Range, b: &Range) -> bool {
    if a.empty || b.empty {
        return false;
    }
    let a_hi = if a.upper_inf { i128::MAX } else { a.upper };
    let b_lo = if b.lower_inf { i128::MIN } else { b.lower };
    a_hi < b_lo
}

pub fn equals(a: &Range, b: &Range) -> bool {
    canonical_eq(a, b)
}

/// Canonical equality (identical point-set after canonicalization).
pub fn canonical_eq(a: &Range, b: &Range) -> bool {
    if a.empty != b.empty {
        return false;
    }
    if a.empty && b.empty {
        return true; // all empty of a type are equal
    }
    // Compare the actual inclusive bounds: lower (inclusive) and the inclusive max
    // (upper-1 for finite `[)` form; infinity preserved). This is the canonical
    // point-set, independent of how the bounds were originally written.
    let a_lo = if a.lower_inf { i128::MIN } else { a.lower };
    let a_hi = if a.upper_inf {
        i128::MAX
    } else if a.upper_inc {
        a.upper
    } else {
        a.upper - 1
    };
    let b_lo = if b.lower_inf { i128::MIN } else { b.lower };
    let b_hi = if b.upper_inf {
        i128::MAX
    } else if b.upper_inc {
        b.upper
    } else {
        b.upper - 1
    };
    a_lo == b_lo && a_hi == b_hi
}

pub fn intersect(a: &Range, b: &Range) -> Range {
    if a.empty || b.empty || !overlaps(a, b) {
        return Range::empty();
    }
    let lo = if a.lower_inf {
        b.lower
    } else if b.lower_inf {
        a.lower
    } else {
        a.lower.max(b.lower)
    };
    let hi = if a.upper_inf {
        b.upper
    } else if b.upper_inf {
        a.upper
    } else {
        a.upper.min(b.upper)
    };
    let lower_inf = a.lower_inf && b.lower_inf;
    let upper_inf = a.upper_inf && b.upper_inf;
    Range {
        empty: false,
        lower_inf,
        upper_inf,
        lower_inc: true,
        upper_inc: false,
        lower: lo,
        upper: hi,
    }
}

/// Minimal enclosing interval; always defined.
pub fn merge(a: &Range, b: &Range) -> Range {
    if a.empty && b.empty {
        return Range::empty(); // canonical clean empty
    }
    if a.empty {
        return *b;
    }
    if b.empty {
        return *a;
    }
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
        lower_inf: a.lower_inf && b.lower_inf,
        upper_inf: a.upper_inf && b.upper_inf,
        lower_inc: true,
        upper_inc: false,
        lower: lo,
        upper: hi,
    }
}

/// Difference a \ b. Returns up to two pieces (FR-8.4: never a single lossy range).
pub fn difference(a: &Range, b: &Range) -> Vec<Range> {
    if a.empty {
        return vec![]; // empty \ anything = empty (no pieces)
    }
    if b.empty || !overlaps(a, b) {
        return vec![*a];
    }
    let mut out = Vec::new();
    // Left piece: [a.lo, b.lo)
    let a_lo = if a.lower_inf { i128::MIN } else { a.lower };
    let b_lo = if b.lower_inf { i128::MIN } else { b.lower };
    if a_lo < b_lo {
        out.push(Range {
            empty: false,
            lower_inf: a.lower_inf,
            upper_inf: false,
            lower_inc: true,
            upper_inc: false,
            lower: a_lo,
            upper: b_lo,
        });
    }
    // Right piece: (b.hi, a.hi]
    let a_hi = if a.upper_inf { i128::MAX } else { a.upper };
    let b_hi = if b.upper_inf { i128::MAX } else { b.upper };
    if b_hi < a_hi {
        out.push(Range {
            empty: false,
            lower_inf: false,
            upper_inf: a.upper_inf,
            lower_inc: true,
            upper_inc: false,
            lower: b_hi,
            upper: a_hi,
        });
    }
    if out.is_empty() { vec![] } else { out }
}

/// Length (discrete width or continuous duration in ordinal units). Empty = 0.
pub fn length(r: &Range) -> i128 {
    if r.empty || r.lower_inf || r.upper_inf {
        return 0; // infinite/empty length is 0 (ordinal units); subtypes scale it
    }
    r.upper - r.lower
}

/// Total order for ORDER BY / GROUP BY / DISTINCT (AD-1 range_compare).
/// empty < non-empty; -inf < finite < +inf; then by lower, then upper.
pub fn total_cmp(a: &Range, b: &Range) -> Ordering {
    // empty sorts before non-empty (empty ord (0,0) < any finite range);
    // -inf < finite < +inf via MIN/MAX sentinels; then by lower, then upper.
    let ord = |r: &Range| -> (i128, i128) {
        let lo = if r.lower_inf { i128::MIN } else { r.lower };
        let hi = if r.upper_inf { i128::MAX } else { r.upper };
        (lo, hi)
    };
    ord(a).cmp(&ord(b))
}
