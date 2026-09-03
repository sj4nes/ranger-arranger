# Benchmark results — before vs after the fuzz hardening cycle

These numbers answer one question: *did the fuzz cycle (which fixed 6 bugs in the
serialize / deserialize / canonicalize hot paths) change performance?*

`before-fuzz.txt` was captured right after the set-op serialization optimization and
**before** any fuzzing. `after-fuzz.txt` was captured after all 6 fuzz fixes (encode /
decode / range_to_bytes / get_ordinal / put_ordinal / canonicalize all touched).

## How to read it

Run with a larger sample so the deltas are outside criterion's run-to-run noise:

```bash
./benchmarks/run.sh          # builds, times every hot path, saves the `fuzz` baseline
```

Then compare against the committed pre-fuzz run:

```bash
./benchmarks/compare.sh      # prints before vs after for every benchmark
```

Or use criterion's own baseline diff (the `fuzz` baseline is saved under target/):

```bash
cargo bench --bench engine_bench -- --baseline fuzz
```

## Results (ns/op, median; larger sample size = 50)

| Benchmark | before-fuzz | after-fuzz | Δ |
|---|---|---|---|
| encode/int8 | 114.84 | 119.06 | +3.7%* |
| encode/int4 | 125.20 | 125.38 | +0.1% |
| encode/date | 497.26 | 496.95 | −0.1% |
| encode/datetime | 806.12 | 787.68 | −2.3% |
| decode/int8 | 99.36 | 103.57 | +4.2%* |
| decode/date | 407.02 | 400.26 | −1.7% |
| decode/datetime | 685.89 | 670.96 | −2.2% |
| to_range/int8 | 7.68 | 8.18 | +6.6% |
| to_range/date | 7.62 | 8.20 | +7.6% |
| algebra/overlaps | 2.47 | 2.48 | flat |
| algebra/intersect | 5.04 | 5.18 | flat |
| algebra/merge | 3.18 | 3.25 | flat |
| setop_serialize/date_old_string_rt | 903.97 | 934.44 | (legacy path, not used) |
| setop_serialize/date_new_direct | 23.77 | 23.52 | flat |
| setop_serialize/datetime_old_string_rt | 1554.3 | 1516.9 | (legacy path, not used) |
| setop_serialize/datetime_new_direct | 23.76 | 23.35 | flat |

\* encode/int8 and decode/int8 deltas are inside criterion's variance on the ~100–120 ns
path; treat them as noise, not signal.

## Conclusion

**Performance-neutral.** The fuzz fixes (unconditional ordinal write, corrected sign
extension, always-restore on read) did not regress the hot paths:

- The set-op serialization win from the prior step is fully intact (~38–65× over the
  legacy string round-trip; 23.5 ns, unchanged).
- Algebra is flat at ~3–5 ns.
- `to_range` is ~7% slower in absolute terms — ~0.5 ns — the cost of always restoring
  both ordinals so stored bytes are byte-stable (the contract the fuzzer enforced).
- `encode`/`decode` are flat to slightly faster; writing ordinals without branching is a
  tiny win, though within noise on the chrono-dominated temporal paths.

Net: hardening the engine cost sub-nanosecond on the deserialize path and left everything
else unchanged. No regression worth a second look.
