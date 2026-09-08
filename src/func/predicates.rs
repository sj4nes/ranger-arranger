// Predicates (FR-6.1-6.7). Per-type so the engine decodes bytes with the right
// endpoint width. AD-4: func! over custom!, NULL-explicit, deterministic: true.
use crate::engine::Range;
use crate::engine::RangeSubtypeOps;
use crate::engine::canonical::to_range;
use crate::subtype;
use villagesql::{InValue, VdfReturn};

type BinaryOp = fn(&Range, &Range) -> bool;

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

/// Build a unary flag predicate `NAME(a custom) -> INT (0/1)`.
pub fn pred_flag<T: RangeSubtypeOps>(flag: fn(&Range) -> bool) -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        match args.first() {
            Some(InValue::Custom(a)) => match to_range::<T>(a) {
                Ok(r) => VdfReturn::int(if flag(&r) { 1 } else { 0 }),
                Err(e) => VdfReturn::error(e),
            },
            Some(InValue::Null) => VdfReturn::null(),
            _ => VdfReturn::error("range flag predicate: expected (custom)"),
        }
    }
}

#[allow(dead_code)]
pub fn int8_contains_point(args: &[InValue]) -> VdfReturn {
    let point = match args.get(1) {
        Some(InValue::Int(p)) => *p,
        _ => return VdfReturn::null(),
    };
    let range_buf = match args.first() {
        Some(InValue::Custom(a)) => a,
        _ => return VdfReturn::null(),
    };
    match to_range::<subtype::int8::Int8Ops>(range_buf) {
        Ok(range) => VdfReturn::int(
            if range.empty || !crate::engine::contains_point(&range, point as i128) {
                0
            } else {
                1
            },
        ),
        Err(_) => VdfReturn::null(),
    }
}

#[allow(dead_code)]
pub fn int8_contains_element(args: &[InValue]) -> VdfReturn {
    let element_buf = match args.get(1) {
        Some(InValue::Custom(b)) => b,
        _ => return VdfReturn::null(),
    };
    let range_buf = match args.first() {
        Some(InValue::Custom(a)) => a,
        _ => return VdfReturn::null(),
    };
    match crate::multirange_types::int8mr_contains_element(range_buf, element_buf) {
        Ok(contained) => VdfReturn::int(if contained { 1 } else { 0 }),
        Err(_) => VdfReturn::null(),
    }
}

#[allow(dead_code)]
pub fn int4_contains_point(args: &[InValue]) -> VdfReturn {
    let point = match args.get(1) {
        Some(InValue::Int(p)) => *p,
        _ => return VdfReturn::null(),
    };
    let range_buf = match args.first() {
        Some(InValue::Custom(a)) => a,
        _ => return VdfReturn::null(),
    };
    match to_range::<subtype::int4::Int4Ops>(range_buf) {
        Ok(range) => VdfReturn::int(
            if range.empty || !crate::engine::contains_point(&range, point as i128) {
                0
            } else {
                1
            },
        ),
        Err(_) => VdfReturn::null(),
    }
}

#[allow(dead_code)]
pub fn int4_contains_element(args: &[InValue]) -> VdfReturn {
    let element_buf = match args.get(1) {
        Some(InValue::Custom(b)) => b,
        _ => return VdfReturn::null(),
    };
    let range_buf = match args.first() {
        Some(InValue::Custom(a)) => a,
        _ => return VdfReturn::null(),
    };
    match crate::multirange_types::int4mr_contains_element(range_buf, element_buf) {
        Ok(contained) => VdfReturn::int(if contained { 1 } else { 0 }),
        Err(_) => VdfReturn::null(),
    }
}

#[allow(dead_code)]
pub fn date_contains_point(args: &[InValue]) -> VdfReturn {
    let point = match args.get(1) {
        Some(InValue::Int(p)) => *p,
        _ => return VdfReturn::null(),
    };
    let range_buf = match args.first() {
        Some(InValue::Custom(a)) => a,
        _ => return VdfReturn::null(),
    };
    match to_range::<subtype::date::DateOps>(range_buf) {
        Ok(range) => VdfReturn::int(
            if range.empty || !crate::engine::contains_point(&range, point as i128) {
                0
            } else {
                1
            },
        ),
        Err(_) => VdfReturn::null(),
    }
}

#[allow(dead_code)]
pub fn date_contains_element(args: &[InValue]) -> VdfReturn {
    let element_buf = match args.get(1) {
        Some(InValue::Custom(b)) => b,
        _ => return VdfReturn::null(),
    };
    let range_buf = match args.first() {
        Some(InValue::Custom(a)) => a,
        _ => return VdfReturn::null(),
    };
    match crate::multirange_types::datemr_contains_element(range_buf, element_buf) {
        Ok(contained) => VdfReturn::int(if contained { 1 } else { 0 }),
        Err(_) => VdfReturn::null(),
    }
}

#[allow(dead_code)]
pub fn datetime_contains_point(args: &[InValue]) -> VdfReturn {
    let point = match args.get(1) {
        Some(InValue::Int(p)) => *p,
        _ => return VdfReturn::null(),
    };
    let range_buf = match args.first() {
        Some(InValue::Custom(a)) => a,
        _ => return VdfReturn::null(),
    };
    match to_range::<subtype::datetime::DateTimeOps>(range_buf) {
        Ok(range) => VdfReturn::int(
            if range.empty || !crate::engine::contains_point(&range, point as i128) {
                0
            } else {
                1
            },
        ),
        Err(_) => VdfReturn::null(),
    }
}

#[allow(dead_code)]
pub fn datetime_contains_element(args: &[InValue]) -> VdfReturn {
    let element_buf = match args.get(1) {
        Some(InValue::Custom(b)) => b,
        _ => return VdfReturn::null(),
    };
    let range_buf = match args.first() {
        Some(InValue::Custom(a)) => a,
        _ => return VdfReturn::null(),
    };
    match crate::multirange_types::dtmr_contains_element(range_buf, element_buf) {
        Ok(contained) => VdfReturn::int(if contained { 1 } else { 0 }),
        Err(_) => VdfReturn::null(),
    }
}
