#![feature(portable_simd)]

const NEWLINES: std::simd::u8x32 = std::simd::u8x32::splat(b'\n');

fn main() {
    let path = std::env::args().nth(1).expect("Provide a path");
    let mut file = std::fs::File::open(path).expect("path should be readable");
    let mut buf = vec![0; 65_536];

    let mut lines = 0;
    loop {
        let mut copyback_len = 0;
        match std::io::Read::read(&mut file, &mut buf[copyback_len..]) {
            Ok(0) => break,
            Ok(n) => {
                let buf = &mut buf[..n + copyback_len];
                let (chunks, remainder) = buf.as_chunks::<32>();
                for chunk in chunks {
                    let chunk_simd = std::simd::u8x32::from(*chunk);
                    let mask = std::simd::cmp::SimdPartialEq::simd_eq(chunk_simd, NEWLINES);
                    let bitmask = mask.to_bitmask();
                    lines += bitmask.count_ones();
                    match bitmask.trailing_zeros() as usize {
                        n if n == chunk.len() => copyback_len += chunk.len(),
                        n => copyback_len = n,
                    }
                }
                let chunk_simd = std::simd::u8x32::load_or_default(remainder);
                let mask = std::simd::cmp::SimdPartialEq::simd_eq(chunk_simd, NEWLINES);
                let bitmask = mask.to_bitmask();
                lines += bitmask.count_ones();
                match bitmask.trailing_zeros() as usize {
                    n if n == remainder.len() => copyback_len += remainder.len(),
                    n => copyback_len = n,
                }
                let copyback_start = buf.len() - copyback_len;
                buf.copy_within(copyback_start.., 0);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("Failed read: {e:?}"),
        }
    }
    println!("{}", lines);
}

#[cfg_attr(not(test), expect(unused))]
fn parse_fixed(input: [u8; 5]) -> i16 {
    let frac = (input[4] - b'0') as i16;
    let ones = (input[2] - b'0') as i16;
    let tens = (input[1] - b'0') as i16;
    let num = frac + 10 * ones + 100 * tens;
    // We use the ASCII character after '+' to indicate negative so that input[0] - b'+' is 0 if
    // the number is positive (hence, we multiply by zero and don't add anything to change the
    // sign) or 1 if the number is negative (hence, we add negative twice the number to make it
    // negative.
    let sign = -(num << 1) * (input[0] - b'+') as i16;

    num + sign
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixed() {
        dbg!((-20_i16).wrapping_sub(20_i16));
        let input = [b'+', b'0', b'1', b'.', b'0'];
        assert_eq!(parse_fixed(input), 10);

        let input = [b'+' + 1 /* negative */, b'0', b'2', b'.', b'0'];
        assert_eq!(parse_fixed(input), -20);

        let input = [b'+', b'3', b'0', b'.', b'1'];
        assert_eq!(parse_fixed(input), 301);
    }
}
