// libFuzzer target: differential algebra fuzzing. Generates two `Range`s via
// `arbitrary` and checks the engine against the brute-force set oracle in
// `fuzz_algebra`. Mirrors `fuzz_algebra` in `src/fuzz_api.rs`.
// Run: cargo +nightly fuzz run algebra
#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use vsql_ranger_arranger::engine::Range;
use vsql_ranger_arranger::fuzz_api::fuzz_algebra;

#[derive(Arbitrary, Debug)]
struct FuzzRange {
    empty: bool,
    lower_inf: bool,
    upper_inf: bool,
    lower_inc: bool,
    upper_inc: bool,
    lower: i64,
    upper: i64,
}

impl From<FuzzRange> for Range {
    fn from(f: FuzzRange) -> Range {
        Range {
            empty: f.empty,
            lower_inf: f.lower_inf,
            upper_inf: f.upper_inf,
            lower_inc: f.lower_inc,
            upper_inc: f.upper_inc,
            lower: f.lower as i128,
            upper: f.upper as i128,
        }
    }
}

fuzz_target!(|data: (FuzzRange, FuzzRange)| {
    let (a, b) = data;
    fuzz_algebra(&a.into(), &b.into());
});
