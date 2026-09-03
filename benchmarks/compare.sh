#!/usr/bin/env bash
# Print a before(vs-fuzz) / after(fuzz) comparison for every benchmark from the
# committed result files. Pure awk — no cargo needed.
set -euo pipefail
cd "$(dirname "$0")"

# A timing line looks like:  encode/int8   time:   [115.84 ns 119.06 ns 123.60 ns]
#   $1 = label, $2 = "time:", $3 = "[low", $5 = median, $6 = unit (ns or µs])
pick() {
  awk '$2 == "time:" {
    label=$1; med=$5; gsub(/[\[\]]/,"",med);
    if ($6 ~ /µs/) med = med * 1000;   # µs -> ns so columns line up
    printf "%s %s\n", label, med
  }' "$1"
}

join -j1 <(pick before-fuzz.txt | sort) <(pick after-fuzz.txt | sort) \
  | awk '{ printf "%-38s before=%-10s after=%-10s\n", $1, $2, $3 }'
