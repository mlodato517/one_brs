#![feature(portable_simd)]

const NEWLINES: std::simd::u8x32 = std::simd::u8x32::splat(b'\n');

fn main() {
    let path = std::env::args().nth(1).expect("Provide a path");
    let mut file = std::fs::File::open(path).expect("path should be readable");
    let mut buf = vec![0; 65_536];

    let mut lines = 0;
    loop {
        match std::io::Read::read(&mut file, &mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let buf = &buf[..n];
                let (chunks, remainder) = buf.as_chunks::<32>();
                for chunk in chunks {
                    let chunk_simd = std::simd::u8x32::from(*chunk);
                    let mask = std::simd::cmp::SimdPartialEq::simd_eq(chunk_simd, NEWLINES);
                    let bitmask = mask.to_bitmask();
                    lines += bitmask.count_ones();
                }
                let chunk_simd = std::simd::u8x32::load_or_default(remainder);
                let mask = std::simd::cmp::SimdPartialEq::simd_eq(chunk_simd, NEWLINES);
                let bitmask = mask.to_bitmask();
                lines += bitmask.count_ones();
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("Failed read: {e:?}"),
        }
    }
    println!("{}", lines);
}
