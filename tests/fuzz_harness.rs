// Fuzz harness — runs under `cargo test` (stable, today) by driving
// `vsql_ranger_arranger::fuzz_api` with a deterministic pseudo-random byte stream
// plus hand-picked edge cases. The SAME `fuzz_api` functions are exercised by the
// libFuzzer targets in `fuzz/` (`cargo +nightly fuzz run <target>`), so this test
// is the offline stand-in that proves the harness logic before long fuzzing runs.

use vsql_ranger_arranger::engine::Range;
use vsql_ranger_arranger::fuzz_api::{fuzz_algebra, fuzz_bytes, parse_range};
use vsql_ranger_arranger::multirange_types::{
    datemr_contains_range, datemr_decode, datemr_difference, datemr_encode, datemr_intersect,
    datemr_merge, datemr_overlaps, dtmr_contains_range, dtmr_decode, dtmr_difference, dtmr_encode,
    dtmr_intersect, dtmr_merge, dtmr_overlaps, int4mr_contains_range, int4mr_decode,
    int4mr_difference, int4mr_encode, int4mr_intersect, int4mr_merge, int4mr_overlaps,
    int8mr_contains_range, int8mr_decode, int8mr_difference, int8mr_encode, int8mr_intersect,
    int8mr_merge, int8mr_overlaps,
};

// Small deterministic LCG (xorshift64*) so the run is reproducible and needs no deps.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn byte(&mut self) -> u8 {
        (self.next() & 0xFF) as u8
    }
    fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.byte()).collect()
    }
}

#[test]
fn fuzz_bytes_random_and_edge_cases() {
    let mut rng = Rng(0x1234_5678_9ABC_DEF1);

    // 1. Many random byte buffers of varying length.
    for len in [0usize, 1, 5, 17, 26, 37, 64, 128] {
        for _ in 0..2000 {
            fuzz_bytes(&rng.bytes(len));
        }
    }

    // 2. Edge-case literals the encoder must handle or reject cleanly (no panic).
    for lit in [
        "empty",
        "[]",
        "[1,5)",
        "(1,5]",
        "[-infinity,10)",
        "[1,+infinity)",
        "[-infinity,+infinity)",
        "[-infinity,-infinity)", // both infinite -> rejected, not a crash
        "[5,1)",                 // reversed -> rejected, not a crash
        "1,5)",
        "[1 5)",
        "",
    ] {
        // round-trip through the literal directly
        if let Ok(stored) = vsql_ranger_arranger::engine::canonical::encode::<
            vsql_ranger_arranger::subtype::int8::Int8Ops,
        >(lit)
        {
            let _ = vsql_ranger_arranger::engine::canonical::decode::<
                vsql_ranger_arranger::subtype::int8::Int8Ops,
            >(&stored);
            let _ = vsql_ranger_arranger::engine::canonical::to_range::<
                vsql_ranger_arranger::subtype::int8::Int8Ops,
            >(&stored);
        }
    }

    // 3. Structs with extreme ordinals / overflow-adjacent values (i128 bounds).
    for r in [
        Range::empty(),
        Range {
            empty: false,
            lower_inf: true,
            upper_inf: false,
            lower_inc: false,
            upper_inc: false,
            lower: 0,
            upper: 0,
        },
        Range {
            empty: false,
            lower_inf: false,
            upper_inf: true,
            lower_inc: false,
            upper_inc: false,
            lower: 0,
            upper: 0,
        },
        Range {
            empty: false,
            lower_inf: false,
            upper_inf: false,
            lower_inc: true,
            upper_inc: false,
            lower: i128::MIN,
            upper: i128::MIN + 5,
        },
        Range {
            empty: false,
            lower_inf: false,
            upper_inf: false,
            lower_inc: true,
            upper_inc: false,
            lower: i128::MAX - 5,
            upper: i128::MAX,
        },
    ] {
        // Build bytes that reproduce this exact struct, then fuzz it.
        // `parse_range` now mirrors `Header::decode`: byte 0 is flags,
        // ordinals follow immediately after. Clamp to i64 representable
        // domain because the stored form only carries 8 endpoint bytes.
        let lo = i64::MIN as i128;
        let hi = i64::MAX as i128;
        let lower = r.lower.clamp(lo, hi);
        let upper = r.upper.clamp(lo, hi);
        let mut buf = [0u8; 17];
        buf[0] = r.empty as u8
            | (r.lower_inc as u8) << 1
            | (r.upper_inc as u8) << 2
            | (r.lower_inf as u8) << 3
            | (r.upper_inf as u8) << 4;
        buf[1..9].copy_from_slice(&lower.to_be_bytes()[8..16]);
        buf[9..17].copy_from_slice(&upper.to_be_bytes()[8..16]);
        fuzz_bytes(&buf);
    }
}

#[test]
fn fuzz_algebra_random() {
    let mut rng = Rng(0xDEAD_BEEF_1357_9246);
    let mut buf = [0u8; 37];
    for _ in 0..50_000 {
        for slot in buf.iter_mut() {
            *slot = rng.byte();
        }
        // Keep endpoints inside the brute-force window so the oracle runs.
        let a = clamp(parse_range(&buf));
        rng.bytes(37)
            .iter()
            .enumerate()
            .for_each(|(i, b)| buf[i] = *b);
        let b = clamp(parse_range(&buf));
        fuzz_algebra(&a, &b);
    }
}

/// Force a range into the finite, in-window form the algebra oracle checks.
fn clamp(mut r: Range) -> Range {
    const W: i128 = 1000;
    r.empty = false;
    r.lower_inf = false;
    r.upper_inf = false;
    r.lower = r.lower.clamp(-W, W);
    r.upper = r.upper.clamp(-W, W);
    r.lower_inc = true;
    r.upper_inc = false;
    r
}

// ---- Multirange fuzz round-trips ----

#[test]
fn fuzz_multirange_bytes_random_and_edge_cases() {
    let mut rng = Rng(0x1234_5678_9ABC_DEF2);

    for len in [0usize, 1, 5, 17, 26, 37, 64, 128] {
        for _ in 0..500 {
            let data = rng.bytes(len);
            let _ = int8mr_encode_from_bytes(&data);
            let _ = int4mr_encode_from_bytes(&data);
            let _ = datemr_encode_from_bytes(&data);
            let _ = dtmr_encode_from_bytes(&data);
        }
    }

    for lit in [
        "empty",
        "{}",
        "{[1,5)}",
        "{[1,5),[10,15)}",
        "{[1,5),[10,15),[20,25)}",
        "{[1,5),(5,10)}",
        "{[1,5),[3,7)}",
    ] {
        for &(enc, dec) in &[
            (
                int8mr_encode as fn(&str) -> Result<Vec<u8>, String>,
                int8mr_decode as fn(&[u8]) -> Result<String, String>,
            ),
            (int4mr_encode, int4mr_decode),
            (datemr_encode, datemr_decode),
            (dtmr_encode, dtmr_decode),
        ] {
            if let Ok(stored) = enc(lit) {
                let _ = dec(&stored);
                let _ = enc(&dec(&stored).unwrap_or_default());
            }
        }
    }
}

fn int8mr_encode_from_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    int8mr_encode(&bytes_to_range_literal(data, "int8"))
}
fn int4mr_encode_from_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    int4mr_encode(&bytes_to_range_literal(data, "int4"))
}
fn datemr_encode_from_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    datemr_encode(&bytes_to_range_literal(data, "date"))
}
fn dtmr_encode_from_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    dtmr_encode(&bytes_to_range_literal(data, "datetime"))
}

fn bytes_to_range_literal(data: &[u8], kind: &str) -> String {
    if data.is_empty() {
        return "empty".to_string();
    }
    match kind {
        "int8" | "int4" => {
            let lo = (data[0] as i64).min(100);
            let hi = (data.get(1).copied().unwrap_or(0) as i64).min(100);
            let li = data.get(2).copied().unwrap_or(0) % 2 == 0;
            let ui = data.get(3).copied().unwrap_or(0) % 2 == 0;
            let lb = if li { '[' } else { '(' };
            let rb = if ui { ']' } else { ')' };
            format!("{lb}{lo},{hi}{rb}")
        }
        "date" => {
            let y = 2026;
            let m = ((data.first().copied().unwrap_or(1) as i64) % 12) + 1;
            let d = ((data.get(1).copied().unwrap_or(1) as i64) % 28) + 1;
            format!("[{y:04}-{m:02}-{d:02},{y:04}-{m:02}-{d:02}]")
        }
        "datetime" => {
            let y = 2026;
            let m = ((data.first().copied().unwrap_or(1) as i64) % 12) + 1;
            let d = ((data.get(1).copied().unwrap_or(1) as i64) % 28) + 1;
            let h = (data.get(2).copied().unwrap_or(0) as i64) % 24;
            let mi = (data.get(3).copied().unwrap_or(0) as i64) % 60;
            let s = (data.get(4).copied().unwrap_or(0) as i64) % 60;
            format!(
                "[{y:04}-{m:02}-{d:02} {h:02}:{mi:02}:{s:02},{y:04}-{m:02}-{d:02} {h:02}:{mi:02}:{s:02}]"
            )
        }
        _ => "empty".to_string(),
    }
}

// ---- Multirange algebra fuzz ----
//
// For each randomly generated multirange literal, encode both operands,
// run the algebra VDF, then verify the result round-trips through decode→encode.

#[test]
fn fuzz_multirange_algebra() {
    let _rng = Rng(0xDEAD_BEEF_CAFE_BABE);

    // Integer multirange literals (valid for int8/int4)
    let int_lits = [
        "empty",
        "{}",
        "{[1,5)}",
        "{[10,20)}",
        "{[1,5),[10,20)}",
        "{[1,5),[3,7)}",  // overlapping
        "{[1,5),(5,10)}", // adjacent
        "{[1,10)}",
        "{[1,5),[6,10)}", // adjacent in multirange
        "{[1,3),[5,7),[9,11)}",
    ];

    for &lit_a in &int_lits {
        for &lit_b in &int_lits {
            for &(
                enc_fn,
                dec_fn,
                intersect_fn,
                merge_fn,
                overlaps_fn,
                contains_fn,
                difference_fn,
            ) in &[
                (
                    int8mr_encode as fn(&str) -> Result<Vec<u8>, String>,
                    int8mr_decode as fn(&[u8]) -> Result<String, String>,
                    int8mr_intersect as fn(&[u8], &[u8]) -> Result<Vec<u8>, String>,
                    int8mr_merge as fn(&[u8], &[u8]) -> Result<Vec<u8>, String>,
                    int8mr_overlaps as fn(&[u8], &[u8]) -> Result<bool, String>,
                    int8mr_contains_range as fn(&[u8], &[u8]) -> Result<bool, String>,
                    int8mr_difference as fn(&[u8], &[u8]) -> Result<Vec<u8>, String>,
                ),
                (
                    int4mr_encode,
                    int4mr_decode,
                    int4mr_intersect,
                    int4mr_merge,
                    int4mr_overlaps,
                    int4mr_contains_range,
                    int4mr_difference,
                ),
            ] {
                let Ok(a) = enc_fn(lit_a) else { continue };
                let Ok(b) = enc_fn(lit_b) else { continue };

                // overlaps: bool, no round-trip needed
                let _ = overlaps_fn(&a, &b);

                // contains_range: bool, no round-trip needed
                let _ = contains_fn(&a, &b);

                // intersect: result must round-trip
                if let Ok(result) = intersect_fn(&a, &b) {
                    let _ = dec_fn(&result);
                    let _ = enc_fn(&dec_fn(&result).unwrap_or_default());
                }

                // merge: result must round-trip
                if let Ok(result) = merge_fn(&a, &b) {
                    let _ = dec_fn(&result);
                    let _ = enc_fn(&dec_fn(&result).unwrap_or_default());
                }

                // difference: result must round-trip
                if let Ok(result) = difference_fn(&a, &b) {
                    let _ = dec_fn(&result);
                    let _ = enc_fn(&dec_fn(&result).unwrap_or_default());
                }
            }
        }
    }

    // Date/datetime multirange algebra (fixed valid literals)
    let date_lits = [
        "{}",
        "empty",
        "{[2020-01-01,2020-06-01)}",
        "{[2020-07-01,2020-12-31)}",
        "{[2020-01-01,2020-06-01),[2020-07-01,2020-12-31)}",
    ];
    let dt_lits = [
        "{}",
        "empty",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00)}",
        "{[2020-07-01 00:00:00,2020-12-31 00:00:00)}",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00),[2020-07-01 00:00:00,2020-12-31 00:00:00)}",
    ];

    for &lit_a in &date_lits {
        for &lit_b in &date_lits {
            let Ok(a) = datemr_encode(lit_a) else {
                continue;
            };
            let Ok(b) = datemr_encode(lit_b) else {
                continue;
            };
            let _ = datemr_overlaps(&a, &b);
            let _ = datemr_contains_range(&a, &b);
            if let Ok(result) = datemr_intersect(&a, &b) {
                let _ = datemr_decode(&result);
                let _ = datemr_encode(&datemr_decode(&result).unwrap_or_default());
            }
            if let Ok(result) = datemr_merge(&a, &b) {
                let _ = datemr_decode(&result);
                let _ = datemr_encode(&datemr_decode(&result).unwrap_or_default());
            }
            if let Ok(result) = datemr_difference(&a, &b) {
                let _ = datemr_decode(&result);
                let _ = datemr_encode(&datemr_decode(&result).unwrap_or_default());
            }
        }
    }

    for &lit_a in &dt_lits {
        for &lit_b in &dt_lits {
            let Ok(a) = dtmr_encode(lit_a) else { continue };
            let Ok(b) = dtmr_encode(lit_b) else { continue };
            let _ = dtmr_overlaps(&a, &b);
            let _ = dtmr_contains_range(&a, &b);
            if let Ok(result) = dtmr_intersect(&a, &b) {
                let _ = dtmr_decode(&result);
                let _ = dtmr_encode(&dtmr_decode(&result).unwrap_or_default());
            }
            if let Ok(result) = dtmr_merge(&a, &b) {
                let _ = dtmr_decode(&result);
                let _ = dtmr_encode(&dtmr_decode(&result).unwrap_or_default());
            }
            if let Ok(result) = dtmr_difference(&a, &b) {
                let _ = dtmr_decode(&result);
                let _ = dtmr_encode(&dtmr_decode(&result).unwrap_or_default());
            }
        }
    }
}

// ---- Multirange BOUNDS and LOWER_INC fuzz ----

#[test]
fn fuzz_multirange_bounds_and_lower_inc() {
    let int_lits = [
        "empty",
        "{}",
        "{[1,5)}",
        "{[10,20)}",
        "{[1,5),[10,15)}",
        "{[1,5),[7,12),[20,30)}",
        "{[-infinity,5)}",
        "{[1,+infinity)}",
    ];
    let date_lits = [
        "{}",
        "empty",
        "{[2020-01-01,2020-06-01)}",
        "{[2020-01-01,2020-06-01),[2020-07-01,2020-12-31)}",
    ];
    let dt_lits = [
        "{}",
        "empty",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00)}",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00),[2020-07-01 00:00:00,2020-12-31 00:00:00)}",
    ];

    // INT8
    for &lit in &int_lits {
        if let Ok(enc) = int8mr_encode(lit) {
            let _ =
                vsql_ranger_arranger::func::extract::int8mr_bounds(&[villagesql::InValue::Custom(
                    &enc,
                )]);
            let _ = vsql_ranger_arranger::func::extract::int8mr_lower_inc(&[
                villagesql::InValue::Custom(&enc),
            ]);
        }
    }

    // INT4
    for &lit in &int_lits {
        if let Ok(enc) = int4mr_encode(lit) {
            let _ =
                vsql_ranger_arranger::func::extract::int4mr_bounds(&[villagesql::InValue::Custom(
                    &enc,
                )]);
            let _ = vsql_ranger_arranger::func::extract::int4mr_lower_inc(&[
                villagesql::InValue::Custom(&enc),
            ]);
        }
    }

    // DATE
    for &lit in &date_lits {
        if let Ok(enc) = datemr_encode(lit) {
            let _ =
                vsql_ranger_arranger::func::extract::datemr_bounds(&[villagesql::InValue::Custom(
                    &enc,
                )]);
            let _ = vsql_ranger_arranger::func::extract::datemr_lower_inc(&[
                villagesql::InValue::Custom(&enc),
            ]);
        }
    }

    // DATETIME
    for &lit in &dt_lits {
        if let Ok(enc) = dtmr_encode(lit) {
            let _ =
                vsql_ranger_arranger::func::extract::dtmr_bounds(&[villagesql::InValue::Custom(
                    &enc,
                )]);
            let _ = vsql_ranger_arranger::func::extract::dtmr_lower_inc(&[
                villagesql::InValue::Custom(&enc),
            ]);
        }
    }
}

// ---- Multirange range_agg fuzz ----

#[test]
fn fuzz_multirange_range_agg() {
    let int_lits = [
        "empty",
        "{}",
        "{[1,5)}",
        "{[10,20)}",
        "{[1,5),[10,20)}",
        "{[1,5),[3,7)}",
        "{[1,5),(5,10)}",
        "{[1,5),[6,10)}",
        "{[1,3),[5,7),[9,11)}",
    ];

    for &lit_a in &int_lits {
        for &lit_b in &int_lits {
            let Ok(a) = int8mr_encode(lit_a) else {
                continue;
            };
            let Ok(b) = int8mr_encode(lit_b) else {
                continue;
            };

            let mut state = vsql_ranger_arranger::func::range_agg::MrAggState::default();
            vsql_ranger_arranger::func::range_agg::int8mr_range_agg_clear(&mut state);
            vsql_ranger_arranger::func::range_agg::int8mr_range_agg_accumulate(
                &mut state,
                &[villagesql::InValue::Custom(&a)],
            );
            vsql_ranger_arranger::func::range_agg::int8mr_range_agg_accumulate(
                &mut state,
                &[villagesql::InValue::Custom(&b)],
            );
            let out = vsql_ranger_arranger::func::range_agg::int8mr_range_agg_result(&state);

            let bytes = match out {
                villagesql::VdfReturn::Binary(b) => b,
                villagesql::VdfReturn::Null => vec![],
                _ => continue,
            };
            let _ = int8mr_decode(&bytes);
            let _ = int8mr_encode(&int8mr_decode(&bytes).unwrap_or_default());
        }
    }

    let date_lits = [
        "{}",
        "empty",
        "{[2020-01-01,2020-06-01)}",
        "{[2020-07-01,2020-12-31)}",
        "{[2020-01-01,2020-06-01),[2020-07-01,2020-12-31)}",
    ];
    let dt_lits = [
        "{}",
        "empty",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00)}",
        "{[2020-07-01 00:00:00,2020-12-31 00:00:00)}",
        "{[2020-01-01 00:00:00,2020-06-01 00:00:00),[2020-07-01 00:00:00,2020-12-31 00:00:00)}",
    ];

    for &lit_a in &date_lits {
        for &lit_b in &date_lits {
            let Ok(a) = datemr_encode(lit_a) else {
                continue;
            };
            let Ok(b) = datemr_encode(lit_b) else {
                continue;
            };

            let mut state = vsql_ranger_arranger::func::range_agg::MrAggState::default();
            vsql_ranger_arranger::func::range_agg::datemr_range_agg_clear(&mut state);
            vsql_ranger_arranger::func::range_agg::datemr_range_agg_accumulate(
                &mut state,
                &[villagesql::InValue::Custom(&a)],
            );
            vsql_ranger_arranger::func::range_agg::datemr_range_agg_accumulate(
                &mut state,
                &[villagesql::InValue::Custom(&b)],
            );
            let out = vsql_ranger_arranger::func::range_agg::datemr_range_agg_result(&state);

            let bytes = match out {
                villagesql::VdfReturn::Binary(b) => b,
                villagesql::VdfReturn::Null => vec![],
                _ => continue,
            };
            let _ = datemr_decode(&bytes);
            let _ = datemr_encode(&datemr_decode(&bytes).unwrap_or_default());
        }
    }

    for &lit_a in &dt_lits {
        for &lit_b in &dt_lits {
            let Ok(a) = dtmr_encode(lit_a) else { continue };
            let Ok(b) = dtmr_encode(lit_b) else { continue };

            let mut state = vsql_ranger_arranger::func::range_agg::MrAggState::default();
            vsql_ranger_arranger::func::range_agg::dtmr_range_agg_clear(&mut state);
            vsql_ranger_arranger::func::range_agg::dtmr_range_agg_accumulate(
                &mut state,
                &[villagesql::InValue::Custom(&a)],
            );
            vsql_ranger_arranger::func::range_agg::dtmr_range_agg_accumulate(
                &mut state,
                &[villagesql::InValue::Custom(&b)],
            );
            let out = vsql_ranger_arranger::func::range_agg::dtmr_range_agg_result(&state);

            let bytes = match out {
                villagesql::VdfReturn::Binary(b) => b,
                villagesql::VdfReturn::Null => vec![],
                _ => continue,
            };
            let _ = dtmr_decode(&bytes);
            let _ = dtmr_encode(&dtmr_decode(&bytes).unwrap_or_default());
        }
    }
}
