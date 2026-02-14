fn main() {
    let path = std::env::args().nth(1).expect("Provide a path");
    let mut file = std::fs::File::open(path).expect("path should be readable");
    let mut buf = Buffer::new(65_536);

    let mut line_count = 0;
    loop {
        match buf.read(&mut file) {
            Ok(0) => break,
            Ok(_) => {
                let read_data = buf.buf();
                let mut last = 0;
                for (i, _b) in read_data.iter().enumerate().filter(|&(_i, b)| *b == b'\n') {
                    let _line = &read_data[last..i];
                    last = i + 1;
                    line_count += 1;
                }
                buf.rollover_at(last);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("Failed read: {e:?}"),
        }
    }
    println!("{}", line_count);
}

struct Buffer {
    /// Buffer of bytes.
    buf: Vec<u8>,

    /// Header slice reserved for end of previous line.
    header_len: usize,

    /// Length of readable buffer
    buf_len: usize,
}
impl Buffer {
    fn new(capacity: usize) -> Self {
        Self {
            buf: vec![0; capacity],
            header_len: 0,
            buf_len: capacity,
        }
    }

    // TODO Change this result type!
    fn read<R: std::io::Read>(&mut self, mut r: R) -> std::io::Result<usize> {
        loop {
            match r.read(&mut self.buf[self.header_len..]) {
                Ok(0) => return Ok(0),
                Ok(n) => {
                    self.buf_len = n;
                    return Ok(n);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
    }

    // TODO Technically don't need to re-scan header for newline -- we know it's not in there.
    fn buf(&self) -> &[u8] {
        &self.buf[..self.total_len()]
    }

    fn total_len(&self) -> usize {
        self.header_len + self.buf_len
    }

    /// Go from:
    ///
    /// [..........., ..............., ..........,   ...................]
    ///  ^^^^^^^^^^^  ^^^^^^^^^^^^^^^  ^^^^^^^^^^    ^^^^^^^^^^^^^^^^^^^
    ///  prev header  new, read data   new header    potentially garbage
    ///                                ^ header_pos
    ///  ^---------^
    ///  self.header_len
    ///               ^--------------------------^
    ///                 self.buf_len
    ///  ^---------------------------------------^
    ///  self.total_len()
    ///                                ^---------^
    ///                                new_header_len
    ///
    /// To:
    ///
    /// [.........., ...................................................]
    ///  ^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ///  new header  data we can read into
    fn rollover_at(&mut self, header_pos: usize) {
        let new_header_len = self.total_len() - header_pos;
        self.buf
            .copy_within(header_pos..header_pos + new_header_len, 0);
        self.header_len = new_header_len;
        self.buf_len = self.buf.len() - self.header_len;
    }
}
