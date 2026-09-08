// Slice 3c: multirange aggregation (`RANGE_AGG`) stateful helpers.
//
// Each multirange type gets its own aggregate descriptor registered in `lib.rs`
// via `villagesql::agg_func!`. The state and hooks are private to this module;
// only the typed wrapper symbols used by `agg_func!` are exposed.

use crate::engine::RangeSubtypeOps;
use crate::multirange_types;
use crate::subtype;
use villagesql::{InValue, VdfReturn, agg_func, custom};

// ── accumulator ──

#[derive(Default)]
pub struct MrAggState {
    buf: Option<Vec<u8>>,
}

// ── generic hooks ──

fn mr_agg_clear(state: &mut MrAggState) {
    state.buf = None;
}

fn mr_agg_accumulate<T: RangeSubtypeOps>(state: &mut MrAggState, args: &[InValue]) {
    let bytes = match args.first() {
        Some(InValue::Custom(a)) => a,
        Some(InValue::Null) => return,
        _ => return,
    };

    let Ok(new_comps) = multirange_types::mr_decode_to_vec::<T>(bytes) else {
        return;
    };

    state.buf = match state.buf.take() {
        None => multirange_types::mr_encode_components::<T>(&new_comps).ok(),
        Some(cur) => {
            let Ok(cur_comps) = multirange_types::mr_decode_to_vec::<T>(&cur) else {
                return;
            };
            let mut merged = cur_comps;
            merged.extend(new_comps);
            let normalized = match multirange_types::normalize_components::<T>(merged) {
                Ok(n) => n,
                Err(_) => return,
            };
            multirange_types::mr_encode_components::<T>(&normalized).ok()
        }
    };
}

fn mr_agg_result<T: RangeSubtypeOps>(state: &MrAggState) -> VdfReturn {
    match &state.buf {
        Some(bytes) => VdfReturn::binary(bytes.clone()),
        None => VdfReturn::binary(multirange_types::mr_canonical_empty::<T>()),
    }
}

// ── typed wrappers ──

pub fn int8mr_range_agg_clear(state: &mut MrAggState) {
    mr_agg_clear(state)
}
pub fn int8mr_range_agg_accumulate(state: &mut MrAggState, args: &[InValue]) {
    mr_agg_accumulate::<subtype::int8::Int8Ops>(state, args)
}
pub fn int8mr_range_agg_result(state: &MrAggState) -> VdfReturn {
    mr_agg_result::<subtype::int8::Int8Ops>(state)
}

pub fn int4mr_range_agg_clear(state: &mut MrAggState) {
    mr_agg_clear(state)
}
pub fn int4mr_range_agg_accumulate(state: &mut MrAggState, args: &[InValue]) {
    mr_agg_accumulate::<subtype::int4::Int4Ops>(state, args)
}
pub fn int4mr_range_agg_result(state: &MrAggState) -> VdfReturn {
    mr_agg_result::<subtype::int4::Int4Ops>(state)
}

pub fn datemr_range_agg_clear(state: &mut MrAggState) {
    mr_agg_clear(state)
}
pub fn datemr_range_agg_accumulate(state: &mut MrAggState, args: &[InValue]) {
    mr_agg_accumulate::<subtype::date::DateOps>(state, args)
}
pub fn datemr_range_agg_result(state: &MrAggState) -> VdfReturn {
    mr_agg_result::<subtype::date::DateOps>(state)
}

pub fn dtmr_range_agg_clear(state: &mut MrAggState) {
    mr_agg_clear(state)
}
pub fn dtmr_range_agg_accumulate(state: &mut MrAggState, args: &[InValue]) {
    mr_agg_accumulate::<subtype::datetime::DateTimeOps>(state, args)
}
pub fn dtmr_range_agg_result(state: &MrAggState) -> VdfReturn {
    mr_agg_result::<subtype::datetime::DateTimeOps>(state)
}

// ── descriptors registered in lib.rs ──

pub const INT8MULTIRANGE_RANGE_AGG_DESC: villagesql::FuncDescriptor = agg_func!(int8mr_range_agg_result, "int8multirange_range_agg", [custom!("INT8MULTIRANGE")] -> custom!("INT8MULTIRANGE"),
        state: MrAggState,
        clear: int8mr_range_agg_clear,
        accumulate: int8mr_range_agg_accumulate,
        buffer_size: 64,
        deterministic: true);

pub const INT4MULTIRANGE_RANGE_AGG_DESC: villagesql::FuncDescriptor = agg_func!(int4mr_range_agg_result, "int4multirange_range_agg", [custom!("INT4MULTIRANGE")] -> custom!("INT4MULTIRANGE"),
        state: MrAggState,
        clear: int4mr_range_agg_clear,
        accumulate: int4mr_range_agg_accumulate,
        buffer_size: 64,
        deterministic: true);

pub const DATEMULTIRANGE_RANGE_AGG_DESC: villagesql::FuncDescriptor = agg_func!(datemr_range_agg_result, "datemultirange_range_agg", [custom!("DATEMULTIRANGE")] -> custom!("DATEMULTIRANGE"),
        state: MrAggState,
        clear: datemr_range_agg_clear,
        accumulate: datemr_range_agg_accumulate,
        buffer_size: 64,
        deterministic: true);

pub const DATETIMEMULTIRANGE_RANGE_AGG_DESC: villagesql::FuncDescriptor = agg_func!(dtmr_range_agg_result, "datetimemultirange_range_agg", [custom!("DATETIMEMULTIRANGE")] -> custom!("DATETIMEMULTIRANGE"),
        state: MrAggState,
        clear: dtmr_range_agg_clear,
        accumulate: dtmr_range_agg_accumulate,
        buffer_size: 64,
        deterministic: true);
