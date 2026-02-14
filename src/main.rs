fn main() {
    let path = std::env::args().nth(1).expect("Provide a path");
    let mut file = std::fs::File::open(path).expect("path should be readable");

    // Would love to use BufReader with fill_buf() and consume(), but it doesn't let us process the
    // end of a buffer when a station is split across two reads.
    let mut buf = vec![0; 65_536];
    let mut buf_start = 0;

    let mut line_count = 0;
    loop {
        match std::io::Read::read(&mut file, &mut buf[buf_start..]) {
            Ok(0) => break,
            Ok(n) => {
                let buf_end = buf_start + n;
                let read_data = &buf[..buf_end];

                let mut last = 0;
                for (i, _b) in read_data.iter().enumerate().filter(|&(_i, b)| *b == b'\n') {
                    let _line = &read_data[last..i];
                    last = i + 1;
                    line_count += 1;
                }
                buf.copy_within(last..buf_end, 0);
                buf_start = buf_end - last;
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("Failed read: {e:?}"),
        }
    }
    println!("{}", line_count);
}
