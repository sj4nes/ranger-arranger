// ranger-arranger v0.1 — VEF range-type extension.
// Wires the generic engine (src/engine) + four subtypes (src/subtype) + functions
// (src/func) into ONE VEF extension via the villagesql macros. AD-2/AD-9.
// Exactly one `extension!` block per crate (it generates vef_register/unregister).

pub mod engine;
pub mod func;
pub mod fuzz_api;
pub mod multirange_types;
pub mod subtype;

// Bring the custom_type! encode/decode/compare fns into scope as bare idents.
use subtype::date::{decode_date, encode_date};
use subtype::datetime::{decode_datetime, encode_datetime};
use subtype::int4::{decode_int4, encode_int4};
use subtype::int8::{decode_int8, encode_int8};

use villagesql::{InValue, Type, VdfReturn, custom, custom_type, func};

// ---- impl shims: the macro expects free fns; they delegate to the typed modules ----
use engine::overlaps;
use func::construct::{empty_for, make_for};
use func::extract::{
    lower_for, lower_inc_for, multirange_lower, multirange_make, upper_for, upper_inc_for,
};
use func::predicates::{pred_binary, pred_flag};
use func::setops::{
    datemr_contains_range, datemr_difference, datemr_intersect, datemr_merge, datemr_overlaps,
    difference_for, dtmr_contains_range, dtmr_difference, dtmr_intersect, dtmr_merge,
    dtmr_overlaps, int4mr_contains_range, int4mr_difference, int4mr_intersect, int4mr_merge,
    int4mr_overlaps, int8mr_contains_range, int8mr_difference, int8mr_intersect, int8mr_merge,
    int8mr_overlaps, intersect_for, length_for, merge_for, union_for,
};

// Null guard for VDF impl entry points. Every VDF impl checks this before
// delegating to its helper, matching the release checklist requirement that null
// input is rejected at the impl boundary, not only inside helpers.
fn guard_null(args: &[InValue]) -> Option<VdfReturn> {
    if args.iter().any(|v| matches!(v, InValue::Null)) {
        return Some(VdfReturn::null());
    }
    None
}

// INT8RANGE
fn int8_make_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    make_for(encode_int8, "INT8RANGE")(a)
}
fn int8_empty_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    empty_for(encode_int8)(a)
}
fn int8_overlaps_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int8::Int8Ops>(overlaps)(a)
}
fn int8_contains_range_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int8::Int8Ops>(engine::contains_range)(a)
}
fn int8_adjacent_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int8::Int8Ops>(engine::adjacent)(a)
}
fn int8_before_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int8::Int8Ops>(engine::before)(a)
}
fn int8_equals_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int8::Int8Ops>(engine::equals)(a)
}
fn int8_isempty_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_flag::<subtype::int8::Int8Ops>(|r| r.empty)(a)
}
fn int8_lower_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    lower_for::<subtype::int8::Int8Ops>()(a)
}
fn int8_upper_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    upper_for::<subtype::int8::Int8Ops>()(a)
}
fn int8_lower_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    lower_inc_for::<subtype::int8::Int8Ops>()(a)
}
fn int8_upper_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    upper_inc_for::<subtype::int8::Int8Ops>()(a)
}
fn int8_intersect_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    intersect_for::<subtype::int8::Int8Ops>()(a)
}
fn int8_merge_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    merge_for::<subtype::int8::Int8Ops>()(a)
}
fn int8_union_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    union_for::<subtype::int8::Int8Ops>()(a)
}
fn int8_difference_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    difference_for::<subtype::int8::Int8Ops>()(a)
}
fn int8_length_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    length_for::<subtype::int8::Int8Ops>()(a)
}

// INT4RANGE
fn int4_make_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    make_for(encode_int4, "INT4RANGE")(a)
}
fn int4_empty_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    empty_for(encode_int4)(a)
}
fn int4_overlaps_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int4::Int4Ops>(overlaps)(a)
}
fn int4_intersect_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    intersect_for::<subtype::int4::Int4Ops>()(a)
}
fn int4_merge_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    merge_for::<subtype::int4::Int4Ops>()(a)
}
// INT4RANGE full surface
fn int4_contains_range_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int4::Int4Ops>(engine::contains_range)(a)
}
fn int4_adjacent_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int4::Int4Ops>(engine::adjacent)(a)
}
fn int4_before_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int4::Int4Ops>(engine::before)(a)
}
fn int4_equals_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::int4::Int4Ops>(engine::equals)(a)
}
fn int4_isempty_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_flag::<subtype::int4::Int4Ops>(|r| r.empty)(a)
}
fn int4_lower_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    lower_for::<subtype::int4::Int4Ops>()(a)
}
fn int4_upper_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    upper_for::<subtype::int4::Int4Ops>()(a)
}
fn int4_lower_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    lower_inc_for::<subtype::int4::Int4Ops>()(a)
}
fn int4_upper_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    upper_inc_for::<subtype::int4::Int4Ops>()(a)
}
fn int4_union_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    union_for::<subtype::int4::Int4Ops>()(a)
}
fn int4_difference_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    difference_for::<subtype::int4::Int4Ops>()(a)
}
fn int4_length_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    length_for::<subtype::int4::Int4Ops>()(a)
}

// DATERANGE
fn date_make_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    make_for(encode_date, "DATERANGE")(a)
}
fn date_empty_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    empty_for(encode_date)(a)
}
fn date_overlaps_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::date::DateOps>(overlaps)(a)
}
fn date_intersect_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    intersect_for::<subtype::date::DateOps>()(a)
}
fn date_merge_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    merge_for::<subtype::date::DateOps>()(a)
}

// DATETIMERANGE
fn dt_make_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    make_for(encode_datetime, "DATETIMERANGE")(a)
}
fn dt_empty_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    empty_for(encode_datetime)(a)
}
fn dt_overlaps_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::datetime::DateTimeOps>(overlaps)(a)
}
fn dt_intersect_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    intersect_for::<subtype::datetime::DateTimeOps>()(a)
}
fn dt_merge_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    merge_for::<subtype::datetime::DateTimeOps>()(a)
}
// DATERANGE full surface
fn date_contains_range_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::date::DateOps>(engine::contains_range)(a)
}
fn date_adjacent_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::date::DateOps>(engine::adjacent)(a)
}
fn date_before_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::date::DateOps>(engine::before)(a)
}
fn date_equals_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::date::DateOps>(engine::equals)(a)
}
fn date_isempty_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_flag::<subtype::date::DateOps>(|r| r.empty)(a)
}
fn date_lower_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    lower_for::<subtype::date::DateOps>()(a)
}
fn date_upper_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    upper_for::<subtype::date::DateOps>()(a)
}
fn date_lower_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    lower_inc_for::<subtype::date::DateOps>()(a)
}
fn date_upper_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    upper_inc_for::<subtype::date::DateOps>()(a)
}
fn date_union_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    union_for::<subtype::date::DateOps>()(a)
}
fn date_difference_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    difference_for::<subtype::date::DateOps>()(a)
}
fn date_length_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    length_for::<subtype::date::DateOps>()(a)
}
// DATETIMERANGE full surface
fn dt_contains_range_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::datetime::DateTimeOps>(engine::contains_range)(a)
}
fn dt_adjacent_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::datetime::DateTimeOps>(engine::adjacent)(a)
}
fn dt_before_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::datetime::DateTimeOps>(engine::before)(a)
}
fn dt_equals_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_binary::<subtype::datetime::DateTimeOps>(engine::equals)(a)
}
fn dt_isempty_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    pred_flag::<subtype::datetime::DateTimeOps>(|r| r.empty)(a)
}
fn dt_lower_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    lower_for::<subtype::datetime::DateTimeOps>()(a)
}
fn dt_upper_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    upper_for::<subtype::datetime::DateTimeOps>()(a)
}
fn dt_lower_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    lower_inc_for::<subtype::datetime::DateTimeOps>()(a)
}
fn dt_upper_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    upper_inc_for::<subtype::datetime::DateTimeOps>()(a)
}
fn dt_union_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    union_for::<subtype::datetime::DateTimeOps>()(a)
}
fn dt_difference_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    difference_for::<subtype::datetime::DateTimeOps>()(a)
}
fn dt_length_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    length_for::<subtype::datetime::DateTimeOps>()(a)
}

// INT8RANGE hash (AD-2): stable hash of canonical point-set bytes.
fn int8_hash(bytes: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

// INT4RANGE hash: same stable-hash-of-encoded-bytes strategy.
fn int4_hash(bytes: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

// DATERANGE hash: same stable-hash-of-encoded-bytes strategy.
fn date_hash(bytes: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

// DATETIMERANGE hash: same stable-hash-of-encoded-bytes strategy.
fn dt_hash(bytes: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    bytes.hash(&mut h);
    h.finish()
}

// ── multirange impl shims ──

fn int8mr_make_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    multirange_make(multirange_types::int8mr_encode, "INT8MULTIRANGE")(a)
}
fn int8mr_lower_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    multirange_lower(multirange_types::int8mr_decode, "INT8MULTIRANGE")(a)
}

fn int4mr_make_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    multirange_make(multirange_types::int4mr_encode, "INT4MULTIRANGE")(a)
}
fn int4mr_lower_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    multirange_lower(multirange_types::int4mr_decode, "INT4MULTIRANGE")(a)
}

fn datemr_make_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    multirange_make(multirange_types::datemr_encode, "DATEMULTIRANGE")(a)
}
fn datemr_lower_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    multirange_lower(multirange_types::datemr_decode, "DATEMULTIRANGE")(a)
}

fn dtmr_make_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    multirange_make(multirange_types::dtmr_encode, "DATETIMEMULTIRANGE")(a)
}
fn dtmr_lower_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    multirange_lower(multirange_types::dtmr_decode, "DATETIMEMULTIRANGE")(a)
}

// ── Slice 2 continued: BOUNDS, LOWER_INC impl shims ──

fn int8mr_bounds_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    func::extract::int8mr_bounds(a)
}
fn int8mr_lower_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    func::extract::int8mr_lower_inc(a)
}

fn int4mr_bounds_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    func::extract::int4mr_bounds(a)
}
fn int4mr_lower_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    func::extract::int4mr_lower_inc(a)
}

fn datemr_bounds_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    func::extract::datemr_bounds(a)
}
fn datemr_lower_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    func::extract::datemr_lower_inc(a)
}

fn dtmr_bounds_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    func::extract::dtmr_bounds(a)
}
fn dtmr_lower_inc_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    func::extract::dtmr_lower_inc(a)
}

// ── multirange algebra VDF impls ──
fn int8mr_overlaps_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int8mr_overlaps(a)
}
fn int8mr_contains_range_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int8mr_contains_range(a)
}
fn int8mr_intersect_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int8mr_intersect(a)
}
fn int8mr_merge_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int8mr_merge(a)
}
fn int8mr_difference_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int8mr_difference(a)
}

fn int4mr_overlaps_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int4mr_overlaps(a)
}
fn int4mr_contains_range_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int4mr_contains_range(a)
}
fn int4mr_intersect_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int4mr_intersect(a)
}
fn int4mr_merge_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int4mr_merge(a)
}
fn int4mr_difference_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    int4mr_difference(a)
}

fn datemr_overlaps_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    datemr_overlaps(a)
}
fn datemr_contains_range_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    datemr_contains_range(a)
}
fn datemr_intersect_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    datemr_intersect(a)
}
fn datemr_merge_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    datemr_merge(a)
}
fn datemr_difference_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    datemr_difference(a)
}

fn dtmr_overlaps_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    dtmr_overlaps(a)
}
fn dtmr_contains_range_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    dtmr_contains_range(a)
}
fn dtmr_intersect_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    dtmr_intersect(a)
}
fn dtmr_merge_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    dtmr_merge(a)
}
fn dtmr_difference_impl(a: &[InValue]) -> VdfReturn {
    if let Some(ret) = guard_null(a) {
        return ret;
    }
    dtmr_difference(a)
}

villagesql::extension! {
    funcs: [
        // INT8RANGE (full surface) — VEF keys VDFs by (name, arg types), so each
        // subtype gets distinct, PostgreSQL-style typed names (int8range/daterange idiom).
        func!(int8_make_impl, "INT8RANGE_MAKE", [Type::String, Type::String, Type::String] -> custom!("INT8RANGE"), buffer_size: 0, deterministic: true),
        func!(int8_empty_impl, "INT8RANGE_EMPTY", [] -> custom!("INT8RANGE"), buffer_size: 0, deterministic: true),
        func!(int8_overlaps_impl, "INT8RANGE_OVERLAPS", [custom!("INT8RANGE"), custom!("INT8RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8_contains_range_impl, "INT8RANGE_CONTAINS_RANGE", [custom!("INT8RANGE"), custom!("INT8RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8_adjacent_impl, "INT8RANGE_ADJACENT", [custom!("INT8RANGE"), custom!("INT8RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8_before_impl, "INT8RANGE_BEFORE", [custom!("INT8RANGE"), custom!("INT8RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8_equals_impl, "INT8RANGE_EQUALS", [custom!("INT8RANGE"), custom!("INT8RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8_isempty_impl, "INT8RANGE_ISEMPTY", [custom!("INT8RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8_lower_impl, "INT8RANGE_LOWER", [custom!("INT8RANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(int8_upper_impl, "INT8RANGE_UPPER", [custom!("INT8RANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(int8_lower_inc_impl, "INT8RANGE_LOWER_INC", [custom!("INT8RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8_upper_inc_impl, "INT8RANGE_UPPER_INC", [custom!("INT8RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8_intersect_impl, "INT8RANGE_INTERSECT", [custom!("INT8RANGE"), custom!("INT8RANGE")] -> custom!("INT8RANGE"), buffer_size: 0, deterministic: true),
        func!(int8_merge_impl, "INT8RANGE_MERGE", [custom!("INT8RANGE"), custom!("INT8RANGE")] -> custom!("INT8RANGE"), buffer_size: 0, deterministic: true),
        func!(int8_union_impl, "INT8RANGE_UNION", [custom!("INT8RANGE"), custom!("INT8RANGE")] -> custom!("INT8RANGE"), buffer_size: 0, deterministic: true),
        func!(int8_difference_impl, "INT8RANGE_DIFFERENCE", [custom!("INT8RANGE"), custom!("INT8RANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(int8_length_impl, "INT8RANGE_LENGTH", [custom!("INT8RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        // INT4RANGE
        func!(int4_make_impl, "INT4RANGE_MAKE", [Type::String, Type::String, Type::String] -> custom!("INT4RANGE"), buffer_size: 0, deterministic: true),
        func!(int4_empty_impl, "INT4RANGE_EMPTY", [] -> custom!("INT4RANGE"), buffer_size: 0, deterministic: true),
        func!(int4_overlaps_impl, "INT4RANGE_OVERLAPS", [custom!("INT4RANGE"), custom!("INT4RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4_intersect_impl, "INT4RANGE_INTERSECT", [custom!("INT4RANGE"), custom!("INT4RANGE")] -> custom!("INT4RANGE"), buffer_size: 0, deterministic: true),
        func!(int4_merge_impl, "INT4RANGE_MERGE", [custom!("INT4RANGE"), custom!("INT4RANGE")] -> custom!("INT4RANGE"), buffer_size: 0, deterministic: true),
        func!(int4_contains_range_impl, "INT4RANGE_CONTAINS_RANGE", [custom!("INT4RANGE"), custom!("INT4RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4_adjacent_impl, "INT4RANGE_ADJACENT", [custom!("INT4RANGE"), custom!("INT4RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4_before_impl, "INT4RANGE_BEFORE", [custom!("INT4RANGE"), custom!("INT4RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4_equals_impl, "INT4RANGE_EQUALS", [custom!("INT4RANGE"), custom!("INT4RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4_isempty_impl, "INT4RANGE_ISEMPTY", [custom!("INT4RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4_lower_impl, "INT4RANGE_LOWER", [custom!("INT4RANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(int4_upper_impl, "INT4RANGE_UPPER", [custom!("INT4RANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(int4_lower_inc_impl, "INT4RANGE_LOWER_INC", [custom!("INT4RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4_upper_inc_impl, "INT4RANGE_UPPER_INC", [custom!("INT4RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4_union_impl, "INT4RANGE_UNION", [custom!("INT4RANGE"), custom!("INT4RANGE")] -> custom!("INT4RANGE"), buffer_size: 0, deterministic: true),
        func!(int4_difference_impl, "INT4RANGE_DIFFERENCE", [custom!("INT4RANGE"), custom!("INT4RANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(int4_length_impl, "INT4RANGE_LENGTH", [custom!("INT4RANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        // DATERANGE
        func!(date_make_impl, "DATERANGE_MAKE", [Type::String, Type::String, Type::String] -> custom!("DATERANGE"), buffer_size: 0, deterministic: true),
        func!(date_empty_impl, "DATERANGE_EMPTY", [] -> custom!("DATERANGE"), buffer_size: 0, deterministic: true),
        func!(date_overlaps_impl, "DATERANGE_OVERLAPS", [custom!("DATERANGE"), custom!("DATERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(date_contains_range_impl, "DATERANGE_CONTAINS_RANGE", [custom!("DATERANGE"), custom!("DATERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(date_adjacent_impl, "DATERANGE_ADJACENT", [custom!("DATERANGE"), custom!("DATERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(date_before_impl, "DATERANGE_BEFORE", [custom!("DATERANGE"), custom!("DATERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(date_equals_impl, "DATERANGE_EQUALS", [custom!("DATERANGE"), custom!("DATERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(date_isempty_impl, "DATERANGE_ISEMPTY", [custom!("DATERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(date_lower_impl, "DATERANGE_LOWER", [custom!("DATERANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(date_upper_impl, "DATERANGE_UPPER", [custom!("DATERANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(date_lower_inc_impl, "DATERANGE_LOWER_INC", [custom!("DATERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(date_upper_inc_impl, "DATERANGE_UPPER_INC", [custom!("DATERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(date_intersect_impl, "DATERANGE_INTERSECT", [custom!("DATERANGE"), custom!("DATERANGE")] -> custom!("DATERANGE"), buffer_size: 0, deterministic: true),
        func!(date_merge_impl, "DATERANGE_MERGE", [custom!("DATERANGE"), custom!("DATERANGE")] -> custom!("DATERANGE"), buffer_size: 0, deterministic: true),
        func!(date_union_impl, "DATERANGE_UNION", [custom!("DATERANGE"), custom!("DATERANGE")] -> custom!("DATERANGE"), buffer_size: 0, deterministic: true),
        func!(date_difference_impl, "DATERANGE_DIFFERENCE", [custom!("DATERANGE"), custom!("DATERANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(date_length_impl, "DATERANGE_LENGTH", [custom!("DATERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        // DATETIMERANGE
        func!(dt_make_impl, "DATETIMERANGE_MAKE", [Type::String, Type::String, Type::String] -> custom!("DATETIMERANGE"), buffer_size: 0, deterministic: true),
        func!(dt_empty_impl, "DATETIMERANGE_EMPTY", [] -> custom!("DATETIMERANGE"), buffer_size: 0, deterministic: true),
        func!(dt_overlaps_impl, "DATETIMERANGE_OVERLAPS", [custom!("DATETIMERANGE"), custom!("DATETIMERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dt_contains_range_impl, "DATETIMERANGE_CONTAINS_RANGE", [custom!("DATETIMERANGE"), custom!("DATETIMERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dt_adjacent_impl, "DATETIMERANGE_ADJACENT", [custom!("DATETIMERANGE"), custom!("DATETIMERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dt_before_impl, "DATETIMERANGE_BEFORE", [custom!("DATETIMERANGE"), custom!("DATETIMERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dt_equals_impl, "DATETIMERANGE_EQUALS", [custom!("DATETIMERANGE"), custom!("DATETIMERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dt_isempty_impl, "DATETIMERANGE_ISEMPTY", [custom!("DATETIMERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dt_lower_impl, "DATETIMERANGE_LOWER", [custom!("DATETIMERANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(dt_upper_impl, "DATETIMERANGE_UPPER", [custom!("DATETIMERANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(dt_lower_inc_impl, "DATETIMERANGE_LOWER_INC", [custom!("DATETIMERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dt_upper_inc_impl, "DATETIMERANGE_UPPER_INC", [custom!("DATETIMERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dt_intersect_impl, "DATETIMERANGE_INTERSECT", [custom!("DATETIMERANGE"), custom!("DATETIMERANGE")] -> custom!("DATETIMERANGE"), buffer_size: 0, deterministic: true),
        func!(dt_merge_impl, "DATETIMERANGE_MERGE", [custom!("DATETIMERANGE"), custom!("DATETIMERANGE")] -> custom!("DATETIMERANGE"), buffer_size: 0, deterministic: true),
        func!(dt_union_impl, "DATETIMERANGE_UNION", [custom!("DATETIMERANGE"), custom!("DATETIMERANGE")] -> custom!("DATETIMERANGE"), buffer_size: 0, deterministic: true),
        func!(dt_difference_impl, "DATETIMERANGE_DIFFERENCE", [custom!("DATETIMERANGE"), custom!("DATETIMERANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(dt_length_impl, "DATETIMERANGE_LENGTH", [custom!("DATETIMERANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8mr_make_impl, "INT8MULTIRANGE_MAKE", [Type::String] -> custom!("INT8MULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(int8mr_lower_impl, "INT8MULTIRANGE_LOWER", [custom!("INT8MULTIRANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(int4mr_make_impl, "INT4MULTIRANGE_MAKE", [Type::String] -> custom!("INT4MULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(int4mr_lower_impl, "INT4MULTIRANGE_LOWER", [custom!("INT4MULTIRANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(datemr_make_impl, "DATEMULTIRANGE_MAKE", [Type::String] -> custom!("DATEMULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(datemr_lower_impl, "DATEMULTIRANGE_LOWER", [custom!("DATEMULTIRANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(dtmr_make_impl, "DATETIMEMULTIRANGE_MAKE", [Type::String] -> custom!("DATETIMEMULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(dtmr_lower_impl, "DATETIMEMULTIRANGE_LOWER", [custom!("DATETIMEMULTIRANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        // multirange algebra
        func!(int8mr_overlaps_impl, "INT8MULTIRANGE_OVERLAPS", [custom!("INT8MULTIRANGE"), custom!("INT8MULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8mr_contains_range_impl, "INT8MULTIRANGE_CONTAINS_RANGE", [custom!("INT8MULTIRANGE"), custom!("INT8MULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int8mr_intersect_impl, "INT8MULTIRANGE_INTERSECT", [custom!("INT8MULTIRANGE"), custom!("INT8MULTIRANGE")] -> custom!("INT8MULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(int8mr_merge_impl, "INT8MULTIRANGE_MERGE", [custom!("INT8MULTIRANGE"), custom!("INT8MULTIRANGE")] -> custom!("INT8MULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(int8mr_difference_impl, "INT8MULTIRANGE_DIFFERENCE", [custom!("INT8MULTIRANGE"), custom!("INT8MULTIRANGE")] -> custom!("INT8MULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(int4mr_overlaps_impl, "INT4MULTIRANGE_OVERLAPS", [custom!("INT4MULTIRANGE"), custom!("INT4MULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4mr_contains_range_impl, "INT4MULTIRANGE_CONTAINS_RANGE", [custom!("INT4MULTIRANGE"), custom!("INT4MULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4mr_intersect_impl, "INT4MULTIRANGE_INTERSECT", [custom!("INT4MULTIRANGE"), custom!("INT4MULTIRANGE")] -> custom!("INT4MULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(int4mr_merge_impl, "INT4MULTIRANGE_MERGE", [custom!("INT4MULTIRANGE"), custom!("INT4MULTIRANGE")] -> custom!("INT4MULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(int4mr_difference_impl, "INT4MULTIRANGE_DIFFERENCE", [custom!("INT4MULTIRANGE"), custom!("INT4MULTIRANGE")] -> custom!("INT4MULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(datemr_overlaps_impl, "DATEMULTIRANGE_OVERLAPS", [custom!("DATEMULTIRANGE"), custom!("DATEMULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(datemr_contains_range_impl, "DATEMULTIRANGE_CONTAINS_RANGE", [custom!("DATEMULTIRANGE"), custom!("DATEMULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(datemr_intersect_impl, "DATEMULTIRANGE_INTERSECT", [custom!("DATEMULTIRANGE"), custom!("DATEMULTIRANGE")] -> custom!("DATEMULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(datemr_merge_impl, "DATEMULTIRANGE_MERGE", [custom!("DATEMULTIRANGE"), custom!("DATEMULTIRANGE")] -> custom!("DATEMULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(datemr_difference_impl, "DATEMULTIRANGE_DIFFERENCE", [custom!("DATEMULTIRANGE"), custom!("DATEMULTIRANGE")] -> custom!("DATEMULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(dtmr_overlaps_impl, "DATETIMEMULTIRANGE_OVERLAPS", [custom!("DATETIMEMULTIRANGE"), custom!("DATETIMEMULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dtmr_contains_range_impl, "DATETIMEMULTIRANGE_CONTAINS_RANGE", [custom!("DATETIMEMULTIRANGE"), custom!("DATETIMEMULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dtmr_intersect_impl, "DATETIMEMULTIRANGE_INTERSECT", [custom!("DATETIMEMULTIRANGE"), custom!("DATETIMEMULTIRANGE")] -> custom!("DATETIMEMULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(dtmr_merge_impl, "DATETIMEMULTIRANGE_MERGE", [custom!("DATETIMEMULTIRANGE"), custom!("DATETIMEMULTIRANGE")] -> custom!("DATETIMEMULTIRANGE"), buffer_size: 0, deterministic: true),
        func!(dtmr_difference_impl, "DATETIMEMULTIRANGE_DIFFERENCE", [custom!("DATETIMEMULTIRANGE"), custom!("DATETIMEMULTIRANGE")] -> custom!("DATETIMEMULTIRANGE"), buffer_size: 0, deterministic: true),
        // Slice 2 continued: BOUNDS, LOWER_INC
        func!(int8mr_bounds_impl, "INT8MULTIRANGE_BOUNDS", [custom!("INT8MULTIRANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(int8mr_lower_inc_impl, "INT8MULTIRANGE_LOWER_INC", [custom!("INT8MULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(int4mr_bounds_impl, "INT4MULTIRANGE_BOUNDS", [custom!("INT4MULTIRANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(int4mr_lower_inc_impl, "INT4MULTIRANGE_LOWER_INC", [custom!("INT4MULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(datemr_bounds_impl, "DATEMULTIRANGE_BOUNDS", [custom!("DATEMULTIRANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(datemr_lower_inc_impl, "DATEMULTIRANGE_LOWER_INC", [custom!("DATEMULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        func!(dtmr_bounds_impl, "DATETIMEMULTIRANGE_BOUNDS", [custom!("DATETIMEMULTIRANGE")] -> Type::String, buffer_size: 0, deterministic: true),
        func!(dtmr_lower_inc_impl, "DATETIMEMULTIRANGE_LOWER_INC", [custom!("DATETIMEMULTIRANGE")] -> Type::Int, buffer_size: 0, deterministic: true),
        // multirange aggregation
        func::range_agg::INT8MULTIRANGE_RANGE_AGG_DESC,
        func::range_agg::INT4MULTIRANGE_RANGE_AGG_DESC,
        func::range_agg::DATEMULTIRANGE_RANGE_AGG_DESC,
        func::range_agg::DATETIMEMULTIRANGE_RANGE_AGG_DESC,
    ],
    types: [
        custom_type!(
            type_name: "INT8RANGE",
            persisted_length: 17,
            max_decode_buffer_length: 64,
            encode: encode_int8,
            decode: decode_int8,
            compare: int8_compare_ident,
            hash: int8_hash,
        ),
        custom_type!(
            type_name: "INT4RANGE",
            persisted_length: 9,
            max_decode_buffer_length: 64,
            encode: encode_int4,
            decode: decode_int4,
            compare: int4_compare_ident,
            hash: int4_hash,
        ),
        custom_type!(
            type_name: "DATERANGE",
            persisted_length: 17,
            max_decode_buffer_length: 64,
            encode: encode_date,
            decode: decode_date,
            compare: date_compare_ident,
            hash: date_hash,
        ),
        custom_type!(
            type_name: "DATETIMERANGE",
            persisted_length: 17,
            max_decode_buffer_length: 64,
            encode: encode_datetime,
            decode: decode_datetime,
            compare: dt_compare_ident,
            hash: dt_hash,
        ),
        multirange_types::int8mr(),
        multirange_types::int4mr(),
        multirange_types::datemr(),
        multirange_types::dtmr(),
    ],
}

// The custom_type! `compare` arg must be a bare fn ident; forward to the engine's
// single compare (AD-1 single-ownership rule).
fn int8_compare_ident(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    crate::engine::compare::range_compare(a, b)
}
fn int4_compare_ident(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    crate::engine::compare::range_compare(a, b)
}
fn date_compare_ident(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    crate::engine::compare::range_compare(a, b)
}
fn dt_compare_ident(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    crate::engine::compare::range_compare(a, b)
}
