#!/usr/bin/env bash
# Build + time every hot path, save the post-fuzz run as the `fuzz` baseline.
# Reproduces benchmarks/after-fuzz.txt.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo bench --bench engine_bench -- --save-baseline fuzz --sample-size 50 --warm-up-time 2
