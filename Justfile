run:
  RUSTFLAGS="-C target-cpu=native" cargo build --release
  hyperfine "./target/release/one_brs measurements.txt"
