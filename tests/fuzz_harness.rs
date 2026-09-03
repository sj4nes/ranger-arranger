// Fuzz harness — runs under `cargo test` (stable, today) by driving
// `vsql_ranger_arranger::fuzz_api` with a deterministic pseudo-random byte stream
// plus hand-picked edge cases. The SAME `fuzz_api` functions are exercised by the
// libFuzzer targets in `fuzz/` (`cargo +nightly fuzz run <target>`), so this test
// is the offline stand-in that proves the harness logic before long fuzzing runs.

use vsql_ranger_arranger::engine::Range;
use vsql_ranger_arranger::fuzz_api::{fuzz_algebra, fuzz_bytes, parse_range};

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
