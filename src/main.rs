fn main() {
    let path = std::env::args().nth(1).expect("Provide a path");
    let file = std::fs::File::open(path).expect("path should be readable");
    let mut file = std::io::BufReader::with_capacity(65_536, file);

    // Max line len is 107 bytes:
    //   - weather station name is <= 100 bytes
    //   - semicolon
    //   - max temperature len is 5 bytes (-10.0)
    //   - \n
    let mut line = String::with_capacity(107);

    let mut len = 0;
    while let Ok(n) = std::io::BufRead::read_line(&mut file, &mut line) {
        if n == 0 {
            break;
        }
        len += line.len();
        line.clear();
    }
    println!("{}", len);
}
