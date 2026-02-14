run:
  cargo build --release
  hyperfine "./target/release/one_brs measurements.txt"
