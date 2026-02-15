fn main() {
    let path = std::env::args().nth(1).expect("Provide a path");
    let mut file = std::fs::File::open(path).expect("path should be readable");

    // Would love to use BufReader with fill_buf() and consume(), but it doesn't let us process the
    // end of a buffer when a station is split across two reads.
    let mut buf = vec![0; 65_536];
    let mut buf_start = 0;

    let mut line_count = 0;
    loop {
        let n = match std::io::Read::read(&mut file, &mut buf[buf_start..]) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => panic!("Failed read: {e:?}"),
        };
        let buf_end = buf_start + n;
        let read_data = &buf[..buf_end];
        let mut split = read_data.rsplit(|b| *b == b'\n');

        // Count how many bytes belong to a line that is split over this read and the next.
        let num_partial_bytes = split.next().expect("rsplit should return something").len();

        for _line in split {
            line_count += 1;
        }

        // Checking if num_partial_bytes > 0 seems worse than just doing it every time.
        buf.copy_within(buf_end - num_partial_bytes..buf_end, 0);
        buf_start = num_partial_bytes;
    }
    println!("{}", line_count);
}
