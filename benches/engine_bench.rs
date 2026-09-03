// Criterion benchmarks for the ranger-arranger hot paths.
// Measures the per-call cost of the operations the server invokes on every range
// column read / predicate evaluation: encode (literal->bytes), decode (bytes->literal),
// to_range (bytes->model), the in-memory algebra, and set-op result serialization
// (the OLD string round-trip vs the NEW direct byte serialization).
//
// Run: cargo bench
// Compare: cargo bench -- --save-baseline before   (before an optimization)
//         cargo bench -- --baseline before          (after, shows delta)
//
// Note: On macOS, this benchmark will warn if the system is running on battery
// power, as power-saving features can distort results.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vsql_ranger_arranger::engine::canonical::{decode, encode, range_to_bytes, to_range};
use vsql_ranger_arranger::engine::{Range, RangeSubtypeOps, intersect, merge, overlaps};
use vsql_ranger_arranger::subtype;

#[cfg(target_os = "macos")]
fn warn_if_on_battery() {
    if let Ok(output) = std::process::Command::new("pmset")
        .args(["-g", "batt"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("Battery") && !stdout.contains("AC Power") {
            eprintln!(
                "WARNING: Running benchmarks on battery power. Results may be distorted by power savings features. Consider plugging in."
            );
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn warn_if_on_battery() {
    // No-op on non-macOS platforms.
}

fn bench_battery_warning(_: &mut Criterion) {
    warn_if_on_battery();
}

fn bench_encode(c: &mut Criterion) {
    let mut g = c.benchmark_group("encode");
    for (name, lit) in [
        ("int8", "[1,5)"),
        ("int4", "[1,50)"),
        ("date", "[2026-01-01,2026-01-31)"),
        ("datetime", "[2026-01-01 00:00:00,2026-01-02 00:00:00)"),
    ] {
        g.bench_function(name, |b| match name {
            "int8" => b.iter(|| encode::<subtype::int8::Int8Ops>(black_box(lit))),
            "int4" => b.iter(|| encode::<subtype::int4::Int4Ops>(black_box(lit))),
            "date" => b.iter(|| encode::<subtype::date::DateOps>(black_box(lit))),
            _ => b.iter(|| encode::<subtype::datetime::DateTimeOps>(black_box(lit))),
        });
    }
    g.finish();
}

fn bench_decode(c: &mut Criterion) {
    let int8 = encode::<subtype::int8::Int8Ops>("[1,5)").unwrap();
    let date = encode::<subtype::date::DateOps>("[2026-01-01,2026-01-31)").unwrap();
    let dt = encode::<subtype::datetime::DateTimeOps>("[2026-01-01 00:00:00,2026-01-02 00:00:00)")
        .unwrap();
    let mut g = c.benchmark_group("decode");
    g.bench_function("int8", |b| {
        b.iter(|| decode::<subtype::int8::Int8Ops>(black_box(&int8)))
    });
    g.bench_function("date", |b| {
        b.iter(|| decode::<subtype::date::DateOps>(black_box(&date)))
    });
    g.bench_function("datetime", |b| {
        b.iter(|| decode::<subtype::datetime::DateTimeOps>(black_box(&dt)))
    });
    g.finish();
}

fn bench_to_range(c: &mut Criterion) {
    let int8 = encode::<subtype::int8::Int8Ops>("[1,5)").unwrap();
    let date = encode::<subtype::date::DateOps>("[2026-01-01,2026-01-31)").unwrap();
    let mut g = c.benchmark_group("to_range");
    g.bench_function("int8", |b| {
        b.iter(|| to_range::<subtype::int8::Int8Ops>(black_box(&int8)))
    });
    g.bench_function("date", |b| {
        b.iter(|| to_range::<subtype::date::DateOps>(black_box(&date)))
    });
    g.finish();
}

fn bench_algebra(c: &mut Criterion) {
    let ra =
        to_range::<subtype::int8::Int8Ops>(&encode::<subtype::int8::Int8Ops>("[1,5)").unwrap())
            .unwrap();
    let rb =
        to_range::<subtype::int8::Int8Ops>(&encode::<subtype::int8::Int8Ops>("[3,8)").unwrap())
            .unwrap();
    let mut g = c.benchmark_group("algebra");
    g.bench_function("overlaps", |b| {
        b.iter(|| overlaps(black_box(&ra), black_box(&rb)))
    });
    g.bench_function("intersect", |b| {
        b.iter(|| intersect(black_box(&ra), black_box(&rb)))
    });
    g.bench_function("merge", |b| {
        b.iter(|| merge(black_box(&ra), black_box(&rb)))
    });
    g.finish();
}

// OLD set-op result path: Range -> literal string -> encode -> re-parse.
fn old_serialize<T: RangeSubtypeOps>(r: &Range) -> Vec<u8> {
    let lit = if r.empty {
        "empty".to_string()
    } else {
        let lo = if r.lower_inf {
            "-infinity".to_string()
        } else {
            T::from_ordinal(r.lower).expect("finite lower ordinal in bench")
        };
        let hi = if r.upper_inf {
            "+infinity".to_string()
        } else {
            T::from_ordinal(r.upper).expect("finite upper ordinal in bench")
        };
        format!("[{},{}]", lo, hi)
    };
    encode::<T>(&lit).unwrap()
}

fn bench_setop_serialize(c: &mut Criterion) {
    let ri = intersect(
        &to_range::<subtype::date::DateOps>(
            &encode::<subtype::date::DateOps>("[2026-01-01,2026-01-31)").unwrap(),
        )
        .unwrap(),
        &to_range::<subtype::date::DateOps>(
            &encode::<subtype::date::DateOps>("[2026-01-15,2026-02-28)").unwrap(),
        )
        .unwrap(),
    );
    let rd = intersect(
        &to_range::<subtype::datetime::DateTimeOps>(
            &encode::<subtype::datetime::DateTimeOps>("[2026-01-01 00:00:00,2026-01-02 12:00:00)")
                .unwrap(),
        )
        .unwrap(),
        &to_range::<subtype::datetime::DateTimeOps>(
            &encode::<subtype::datetime::DateTimeOps>("[2026-01-01 12:00:00,2026-01-03 00:00:00)")
                .unwrap(),
        )
        .unwrap(),
    );
    let mut g = c.benchmark_group("setop_serialize");
    g.bench_function("date_old_string_rt", |b| {
        b.iter(|| old_serialize::<subtype::date::DateOps>(black_box(&ri)))
    });
    g.bench_function("date_new_direct", |b| {
        b.iter(|| range_to_bytes::<subtype::date::DateOps>(black_box(&ri)))
    });
    g.bench_function("datetime_old_string_rt", |b| {
        b.iter(|| old_serialize::<subtype::datetime::DateTimeOps>(black_box(&rd)))
    });
    g.bench_function("datetime_new_direct", |b| {
        b.iter(|| range_to_bytes::<subtype::datetime::DateTimeOps>(black_box(&rd)))
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_battery_warning,
    bench_encode,
    bench_decode,
    bench_to_range,
    bench_algebra,
    bench_setop_serialize
);
criterion_main!(benches);
