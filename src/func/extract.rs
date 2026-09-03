// Extraction (FR-7.2): RANGE_LOWER/UPPER/BOUNDS + flag accessors. AD-4.
use crate::engine::RangeSubtypeOps;
use crate::engine::canonical::to_range;
use crate::subtype;
use villagesql::{InValue, VdfReturn};

/// RANGE_LOWER(r) -> TEXT (the lower endpoint literal, or '-infinity'/'').
pub fn lower_for<T: RangeSubtypeOps>() -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        match args.first() {
            Some(InValue::Custom(a)) => match to_range::<T>(a) {
                Ok(r) => {
                    if r.empty {
                        VdfReturn::null()
                    } else if r.lower_inf {
                        VdfReturn::string("-infinity")
                    } else {
                        match T::from_ordinal(r.lower) {
                            Ok(s) => VdfReturn::string(s),
                            Err(e) => VdfReturn::error(e),
                        }
                    }
                }
                Err(e) => VdfReturn::error(e),
            },
            Some(InValue::Null) => VdfReturn::null(),
            _ => VdfReturn::error("RANGE_LOWER: expected (custom)"),
        }
    }
}

/// RANGE_UPPER(r) -> TEXT.
pub fn upper_for<T: RangeSubtypeOps>() -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        match args.first() {
            Some(InValue::Custom(a)) => match to_range::<T>(a) {
                Ok(r) => {
                    if r.empty {
                        VdfReturn::null()
                    } else if r.upper_inf {
                        VdfReturn::string("+infinity")
                    } else {
                        match T::from_ordinal(r.upper) {
                            Ok(s) => VdfReturn::string(s),
                            Err(e) => VdfReturn::error(e),
                        }
                    }
                }
                Err(e) => VdfReturn::error(e),
            },
            Some(InValue::Null) => VdfReturn::null(),
            _ => VdfReturn::error("RANGE_UPPER: expected (custom)"),
        }
    }
}

/// RANGE_LOWER_INC / RANGE_UPPER_INC / RANGE_BOUNDS -> INT (0/1).
pub fn lower_inc_for<T: RangeSubtypeOps>() -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        match args.first() {
            Some(InValue::Custom(a)) => match to_range::<T>(a) {
                Ok(r) => VdfReturn::int(if r.empty { 0 } else { r.lower_inc as i64 }),
                Err(e) => VdfReturn::error(e),
            },
            Some(InValue::Null) => VdfReturn::null(),
            _ => VdfReturn::error("RANGE_LOWER_INC: expected (custom)"),
        }
    }
}
pub fn upper_inc_for<T: RangeSubtypeOps>() -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        match args.first() {
            Some(InValue::Custom(a)) => match to_range::<T>(a) {
                Ok(r) => VdfReturn::int(if r.empty { 0 } else { r.upper_inc as i64 }),
                Err(e) => VdfReturn::error(e),
            },
            Some(InValue::Null) => VdfReturn::null(),
            _ => VdfReturn::error("RANGE_UPPER_INC: expected (custom)"),
        }
    }
}

#[allow(dead_code)]
pub fn int8_lower(args: &[InValue]) -> VdfReturn {
    lower_for::<subtype::int8::Int8Ops>()(args)
}
#[allow(dead_code)]
pub fn int8_upper(args: &[InValue]) -> VdfReturn {
    upper_for::<subtype::int8::Int8Ops>()(args)
}
#[allow(dead_code)]
pub fn int8_lower_inc(args: &[InValue]) -> VdfReturn {
    lower_inc_for::<subtype::int8::Int8Ops>()(args)
}
#[allow(dead_code)]
pub fn int8_upper_inc(args: &[InValue]) -> VdfReturn {
    upper_inc_for::<subtype::int8::Int8Ops>()(args)
}
