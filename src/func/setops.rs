// Set operations (FR-8.1-8.5). AD-4: func! over custom!, NULL-explicit.
// intersect/union/merge re-encode the result range; difference returns a JSON
// string of pieces (FR-8.4: never a single lossy range).
use crate::engine::canonical::{range_to_bytes, to_range};
use crate::engine::{RangeSubtypeOps, difference, intersect, length, merge, overlaps};
use crate::subtype;
use villagesql::{InValue, VdfReturn};

type BinaryRangeOp = fn(&crate::engine::Range, &crate::engine::Range) -> crate::engine::Range;

fn bin_op<T: RangeSubtypeOps>(
    args: &[InValue],
    op: BinaryRangeOp,
    disjoint_err: Option<&str>,
) -> VdfReturn {
    match (args.first(), args.get(1)) {
        (Some(InValue::Custom(a)), Some(InValue::Custom(b))) => {
            let ra = match to_range::<T>(a) {
                Ok(r) => r,
                Err(e) => return VdfReturn::error(e),
            };
            let rb = match to_range::<T>(b) {
                Ok(r) => r,
                Err(e) => return VdfReturn::error(e),
            };
            if let Some(msg) = disjoint_err.filter(|_| !overlaps(&ra, &rb)) {
                return VdfReturn::error(msg.to_string());
            }
            VdfReturn::binary(range_to_bytes::<T>(&op(&ra, &rb)))
        }
        (Some(InValue::Null), _) | (_, Some(InValue::Null)) => VdfReturn::null(),
        _ => VdfReturn::error("range set-op: expected (custom, custom)"),
    }
}

/// Serialize a `Range` model back to a literal for re-encoding. Endpoint ordinals
/// are formatted via `T::from_ordinal` so the literal is human-readable and can be
/// re-parsed by `encode::<T>` (e.g. a date ordinal 20468 -> '2026-..', not '20468').
fn range_to_literal<T: RangeSubtypeOps>(r: &crate::engine::Range) -> Result<String, String> {
    if r.empty {
        return Ok("empty".to_string());
    }
    let lb = if r.lower_inf { "(" } else { "[" };
    let rb = if r.upper_inf {
        ")"
    } else if r.upper_inc {
        "]"
    } else {
        ")"
    };
    let lo = if r.lower_inf {
        "-infinity".to_string()
    } else {
        T::from_ordinal(r.lower).map_err(|e| format!("range_to_literal: {e}"))?
    };
    let hi = if r.upper_inf {
        "+infinity".to_string()
    } else {
        T::from_ordinal(r.upper).map_err(|e| format!("range_to_literal: {e}"))?
    };
    Ok(format!("{}{},{}{}", lb, lo, hi, rb))
}

pub fn intersect_for<T: RangeSubtypeOps>() -> impl Fn(&[InValue]) -> VdfReturn {
    |args| bin_op::<T>(args, intersect, None)
}
pub fn merge_for<T: RangeSubtypeOps>() -> impl Fn(&[InValue]) -> VdfReturn {
    |args| bin_op::<T>(args, merge, None)
}
pub fn union_for<T: RangeSubtypeOps>() -> impl Fn(&[InValue]) -> VdfReturn {
    |args| {
        bin_op::<T>(
            args,
            intersect,
            Some("RANGE_UNION: inputs are disjoint; use RANGE_MERGE for enclosing interval"),
        )
    }
}

/// RANGE_DIFFERENCE(a, b) -> JSON string of pieces (FR-8.4 anti-lossy).
pub fn difference_for<T: RangeSubtypeOps>() -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        match (args.first(), args.get(1)) {
            (Some(InValue::Custom(a)), Some(InValue::Custom(b))) => {
                let ra = match to_range::<T>(a) {
                    Ok(r) => r,
                    Err(e) => return VdfReturn::error(e),
                };
                let rb = match to_range::<T>(b) {
                    Ok(r) => r,
                    Err(e) => return VdfReturn::error(e),
                };
                let pieces = difference(&ra, &rb);
                let mut out = String::from("[");
                for (i, p) in pieces.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    match range_to_literal::<T>(p) {
                        Ok(lit) => out.push_str(&lit),
                        Err(e) => return VdfReturn::error(e),
                    }
                }
                out.push(']');
                VdfReturn::string(out)
            }
            (Some(InValue::Null), _) | (_, Some(InValue::Null)) => VdfReturn::null(),
            _ => VdfReturn::error("RANGE_DIFFERENCE: expected (custom, custom)"),
        }
    }
}

/// RANGE_LENGTH(r) -> INT (ordinal width; 0 for empty/infinite).
pub fn length_for<T: RangeSubtypeOps>() -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        match args.first() {
            Some(InValue::Custom(a)) => match to_range::<T>(a) {
                Ok(r) => VdfReturn::int(length(&r) as i64),
                Err(e) => VdfReturn::error(e),
            },
            Some(InValue::Null) => VdfReturn::null(),
            _ => VdfReturn::error("RANGE_LENGTH: expected (custom)"),
        }
    }
}

#[allow(dead_code)]
pub fn int8_intersect(args: &[InValue]) -> VdfReturn {
    intersect_for::<subtype::int8::Int8Ops>()(args)
}
#[allow(dead_code)]
pub fn int8_merge(args: &[InValue]) -> VdfReturn {
    merge_for::<subtype::int8::Int8Ops>()(args)
}
#[allow(dead_code)]
pub fn int8_union(args: &[InValue]) -> VdfReturn {
    union_for::<subtype::int8::Int8Ops>()(args)
}
#[allow(dead_code)]
pub fn int8_difference(args: &[InValue]) -> VdfReturn {
    difference_for::<subtype::int8::Int8Ops>()(args)
}
#[allow(dead_code)]
pub fn int8_length(args: &[InValue]) -> VdfReturn {
    length_for::<subtype::int8::Int8Ops>()(args)
}
