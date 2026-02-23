use std::fs::File;

fn main() {
    let mut args = std::env::args().skip(1);
    let source_path = args.next().unwrap();
    let dest_path = args.next().unwrap();

    let mut source = File::open(source_path).unwrap();
    let mut dest = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
        .unwrap();

    std::io::copy(&mut source, &mut dest).unwrap();
}
