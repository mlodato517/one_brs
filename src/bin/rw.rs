use std::fs::File;
use std::io::{ErrorKind, Read, Write};

fn main() {
    let mut args = std::env::args().skip(1);
    let source_path = args.next().unwrap();
    let dest_path = args.next().unwrap();
    let kibibytes: usize = args.next().unwrap().parse().unwrap();

    let mut source = File::open(source_path).unwrap();
    let mut dest = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
        .unwrap();

    let mut buf = vec![0; kibibytes * 1024];
    loop {
        match source.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => dest.write_all(&buf[..n]).unwrap(),
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => panic!("{e:?}"),
        }
    }
}
