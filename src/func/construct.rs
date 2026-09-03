// Constructors (FR-2.2, FR-7.1). Per-type so the server routes to the right
// custom_type! encode. AD-4: func! over custom!, NULL-explicit.
use crate::subtype;
use villagesql::{InValue, VdfReturn};

/// Generic per-type constructor: build `<TYPE>_MAKE(lower TEXT, upper TEXT, bounds TEXT)`.
/// `enc` encodes a literal for the target subtype.
pub fn make_for(
    enc: fn(&str) -> Result<Vec<u8>, String>,
    type_name: &str,
) -> impl Fn(&[InValue]) -> VdfReturn {
    move |args: &[InValue]| -> VdfReturn {
        let (lo, hi, bounds) = match (args.first(), args.get(1), args.get(2)) {
            (Some(InValue::String(lo)), Some(InValue::String(hi)), Some(InValue::String(b))) => {
                (lo.to_string(), hi.to_string(), b.to_string())
            }
            (Some(InValue::Null), _, _) | (_, Some(InValue::Null), _) => {
                return VdfReturn::null();
            }
            _ => return VdfReturn::error(format!("{type_name}_MAKE: expected (TEXT, TEXT, TEXT)")),
        };
        let mut lit = String::with_capacity(lo.len() + hi.len() + 4);
        lit.push(bounds.chars().next().unwrap_or('['));
        lit.push_str(&lo);
        lit.push(',');
        lit.push_str(&hi);
        lit.push(bounds.chars().nth(1).unwrap_or(')'));
        match enc(&lit) {
            Ok(bytes) => VdfReturn::binary(bytes),
            Err(e) => VdfReturn::error(e),
        }
    }
}

/// RANGE_EMPTY() for each type -> empty range bytes.
pub fn empty_for(enc: fn(&str) -> Result<Vec<u8>, String>) -> impl Fn(&[InValue]) -> VdfReturn {
    move |_args: &[InValue]| -> VdfReturn {
        match enc("empty") {
            Ok(bytes) => VdfReturn::binary(bytes),
            Err(e) => VdfReturn::error(e),
        }
    }
}

#[allow(dead_code)]
pub fn int8_make(args: &[InValue]) -> VdfReturn {
    make_for(subtype::int8::encode_int8, "INT8RANGE")(args)
}
#[allow(dead_code)]
pub fn int8_empty(args: &[InValue]) -> VdfReturn {
    empty_for(subtype::int8::encode_int8)(args)
}
