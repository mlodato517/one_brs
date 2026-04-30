#![feature(portable_simd)]

const NEWLINES: std::simd::u8x32 = std::simd::u8x32::splat(b'\n');
const SEMICOLONS: std::simd::u8x32 = std::simd::u8x32::splat(b';');

fn main() {
    let path = std::env::args().nth(1).expect("Provide a path");
    let mut file = std::fs::File::open(path).expect("path should be readable");
    let mut buf = vec![0; 65_536];

    // [000000000000]
    // [000000010000]
    // [000010000020]
    // 0b00001000000
    // mask1.count_leading_zeros()
    // mask2.count_leading_zeros()
    // [0 0 1 x x 2 x]

    let mut bytes = 0;
    loop {
        match std::io::Read::read(&mut file, &mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let buf = &buf[..n];
                let (chunks, remainder) = buf.as_chunks::<32>();
                for chunk in chunks {
                    let chunk_simd = std::simd::u8x32::from(*chunk);
                    let semicolon_idx =
                        std::simd::cmp::SimdPartialEq::simd_eq(chunk_simd, SEMICOLONS).first_set();
                    let newline_idx =
                        std::simd::cmp::SimdPartialEq::simd_eq(chunk_simd, NEWLINES).first_set();
                    match (semicolon_idx, newline_idx) {
                        (None, None) => {
                            bytes += chunk.len();
                        }
                        (Some(semicolon_idx), None) => {
                            bytes += semicolon_idx;
                            bytes += chunk.len() - semicolon_idx;
                        }
                        (Some(semicolon_idx), Some(newline_idx)) => {
                            bytes += semicolon_idx;
                            bytes += newline_idx - semicolon_idx;
                            bytes += chunk.len() - newline_idx;
                        }
                        _ => unsafe { std::hint::unreachable_unchecked() },
                    }
                    // lines += newline_mask.count_ones();
                }
                let chunk_simd = std::simd::u8x32::load_or_default(remainder);
                let mask = std::simd::cmp::SimdPartialEq::simd_eq(chunk_simd, NEWLINES);
                let bitmask = mask.to_bitmask();
                // lines += bitmask.count_ones();
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("Failed read: {e:?}"),
        }
    }
    // println!("{}", lines);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let chunk: &[u8; 32] = &[
            0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        // 0b00000100000000000000000000000000
        // 0b00000000000000000000000000100000
        const ONE: std::simd::u8x32 = std::simd::u8x32::splat(1);
        let chunk_simd = std::simd::u8x32::from(*chunk);
        let semicolon_mask = std::simd::cmp::SimdPartialEq::simd_eq(chunk_simd, ONE);
        dbg!(semicolon_mask);
        dbg!(semicolon_mask.shift_elements_left::<5>(false));
        dbg!(semicolon_mask.shift_elements_left::<5>(false).first_set());
        panic!();
    }
}
