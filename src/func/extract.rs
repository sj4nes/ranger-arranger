// Extraction (FR-7.2): RANGE_LOWER/UPPER/BOUNDS + flag accessors. AD-4.
use crate::engine::RangeSubtypeOps;
use crate::engine::canonical::to_range;
use crate::multirange_types;
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

// ── multirange constructors/extractors ──

pub fn multirange_make(
    enc: fn(&str) -> Result<Vec<u8>, String>,
    type_name: &str,
) -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        let lit = match args.first() {
            Some(InValue::String(s)) => s,
            Some(InValue::Null) => return VdfReturn::null(),
            _ => {
                return VdfReturn::error(format!(
                    "{type_name}_MAKE: expected (TEXT multirange_literal)"
                ));
            }
        };
        match enc(lit) {
            Ok(bytes) => VdfReturn::binary(bytes),
            Err(e) => VdfReturn::error(e),
        }
    }
}

pub fn multirange_lower(
    dec: fn(&[u8]) -> Result<String, String>,
    type_name: &str,
) -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        match args.first() {
            Some(InValue::Custom(a)) => match dec(a) {
                Ok(s) => VdfReturn::string(s),
                Err(e) => VdfReturn::error(e),
            },
            Some(InValue::Null) => VdfReturn::null(),
            _ => VdfReturn::error(format!("{type_name}_LOWER: expected (custom)")),
        }
    }
}

pub fn int8mr_make(args: &[InValue]) -> VdfReturn {
    multirange_make(multirange_types::int8mr_encode, "INT8MULTIRANGE")(args)
}
pub fn int8mr_lower(args: &[InValue]) -> VdfReturn {
    multirange_lower(multirange_types::int8mr_decode, "INT8MULTIRANGE")(args)
}

pub fn int4mr_make(args: &[InValue]) -> VdfReturn {
    multirange_make(multirange_types::int4mr_encode, "INT4MULTIRANGE")(args)
}
pub fn int4mr_lower(args: &[InValue]) -> VdfReturn {
    multirange_lower(multirange_types::int4mr_decode, "INT4MULTIRANGE")(args)
}

pub fn datemr_make(args: &[InValue]) -> VdfReturn {
    multirange_make(multirange_types::datemr_encode, "DATEMULTIRANGE")(args)
}
pub fn datemr_lower(args: &[InValue]) -> VdfReturn {
    multirange_lower(multirange_types::datemr_decode, "DATEMULTIRANGE")(args)
}

pub fn dtmr_make(args: &[InValue]) -> VdfReturn {
    multirange_make(multirange_types::dtmr_encode, "DATETIMEMULTIRANGE")(args)
}
pub fn dtmr_lower(args: &[InValue]) -> VdfReturn {
    multirange_lower(multirange_types::dtmr_decode, "DATETIMEMULTIRANGE")(args)
}
