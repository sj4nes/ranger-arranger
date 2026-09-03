// Predicates (FR-6.1-6.7). Per-type so the engine decodes bytes with the right
// endpoint width. AD-4: func! over custom!, NULL-explicit, deterministic: true.
use crate::engine::canonical::to_range;
use crate::engine::{RangeSubtypeOps, adjacent, before, contains_range, equals, overlaps};
use crate::subtype;
use villagesql::{InValue, VdfReturn};

type BinaryOp = fn(&crate::engine::Range, &crate::engine::Range) -> bool;

/// Build a binary range predicate `NAME(a custom, b custom) -> INT (0/1)`.
pub fn pred_binary<T: RangeSubtypeOps>(op: BinaryOp) -> impl Fn(&[InValue]) -> VdfReturn {
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
                VdfReturn::int(if op(&ra, &rb) { 1 } else { 0 })
            }
            (Some(InValue::Null), _) | (_, Some(InValue::Null)) => VdfReturn::null(),
            _ => VdfReturn::error("range predicate: expected (custom, custom)"),
        }
    }
}

/// Build a unary flag accessor `NAME(r custom) -> INT (0/1)`.
pub fn pred_flag<T: RangeSubtypeOps>(
    f: fn(&crate::engine::Range) -> bool,
) -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        match args.first() {
            Some(InValue::Custom(a)) => match to_range::<T>(a) {
                Ok(r) => VdfReturn::int(if f(&r) { 1 } else { 0 }),
                Err(e) => VdfReturn::error(e),
            },
            Some(InValue::Null) => VdfReturn::null(),
            _ => VdfReturn::error("range flag: expected (custom)"),
        }
    }
}

#[allow(dead_code)]
pub fn int8_overlaps(args: &[InValue]) -> VdfReturn {
    pred_binary::<subtype::int8::Int8Ops>(overlaps)(args)
}
#[allow(dead_code)]
pub fn int8_contains_range(args: &[InValue]) -> VdfReturn {
    pred_binary::<subtype::int8::Int8Ops>(contains_range)(args)
}
#[allow(dead_code)]
pub fn int8_adjacent(args: &[InValue]) -> VdfReturn {
    pred_binary::<subtype::int8::Int8Ops>(adjacent)(args)
}
#[allow(dead_code)]
pub fn int8_before(args: &[InValue]) -> VdfReturn {
    pred_binary::<subtype::int8::Int8Ops>(before)(args)
}
#[allow(dead_code)]
pub fn int8_equals(args: &[InValue]) -> VdfReturn {
    pred_binary::<subtype::int8::Int8Ops>(equals)(args)
}
#[allow(dead_code)]
pub fn int8_isempty(args: &[InValue]) -> VdfReturn {
    pred_flag::<subtype::int8::Int8Ops>(|r| r.empty)(args)
}
