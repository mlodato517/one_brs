use std::fs::File;

use memmap2::Mmap;

fn main() {
    let mut args = std::env::args().skip(1);
    let source_path = args.next().unwrap();
    let dest_path = args.next().unwrap();

    let source = File::open(source_path).unwrap();
    let source = unsafe { Mmap::map(&source).unwrap() };
    let mut dest = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
        .unwrap();

    std::io::copy(&mut &*source, &mut dest).unwrap();
}
