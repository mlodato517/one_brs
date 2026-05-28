build:
  RUSTFLAGS="-C target-cpu=native" cargo build --release

[default]
run: build
  hyperfine "./target/release/one_brs measurements.txt"
