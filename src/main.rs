fn main() {
    let path = std::env::args().nth(1).expect("Provide a path");
    let mut file = std::fs::File::open(path).expect("path should be readable");
    let mut buf = vec![0; 65_536];

    let mut line_count = 0;
    loop {
        match std::io::Read::read(&mut file, &mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let buf = &buf[..n];
                line_count += buf.iter().filter(|&b| *b == b'\n').count()
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("Failed read: {e:?}"),
        }
    }
    println!("{}", line_count);
}
