#!/bin/sh

# Could use -L, but don't want the KiB param for non-rw_copy.
# Just use 128KiB (it seemed the fastest in local tests)
# and 256KiB (to match `cat`/`cp`).
#
# To drop cache, add:
# ```sh
#  --prepare 'sync; echo 3 | sudo tee /proc/sys/vm/drop_caches' \
# ```
#
# from https://github.com/sharkdp/hyperfine/blob/master/README.md#warmup-runs-and-preparation-commands.
#
# Note that, for me, the Advise didn't seem to matter, so it seems reasonable
# to drop those.
#
# View timings table in HTML with `pandoc`:
#
# ```sh
# pandoc -t html -s experiment_1_timings.md -o experiment_1_timings.html
# ```
hyperfine --warmup 2 \
  --setup "cargo build --release" \
  -L dest measurements2.txt,/dev/null \
  --export-markdown experiment_1_timings.md \
  "cat measurements.txt > {dest}" \
  "cp measurements.txt {dest}" \
  "./target/release/copy measurements.txt {dest}" \
  "./target/release/rw measurements.txt {dest} 128" \
  "./target/release/rw measurements.txt {dest} 256" \
  "./target/release/memmap-copy measurements.txt {dest}" \
  "./target/release/memmap-rw measurements.txt {dest} Full NoAdvise" \
  "./target/release/memmap-rw measurements.txt {dest} Full Sequential" \
  "./target/release/memmap-rw measurements.txt {dest} 128 NoAdvise" \
  "./target/release/memmap-rw measurements.txt {dest} 128 Sequential" \
  "./target/release/memmap-rw measurements.txt {dest} 256 NoAdvise" \
  "./target/release/memmap-rw measurements.txt {dest} 256 Sequential"
