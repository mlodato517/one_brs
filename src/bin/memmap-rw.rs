use std::fs::File;
use std::io::Write;

use memmap2::{Advice, Mmap};

fn main() {
    let mut args = std::env::args().skip(1);
    let source_path = args.next().unwrap();
    let dest_path = args.next().unwrap();

    // If a number, we'll go by chunks. Otherwise, we'll write the whole thing.
    // Very lazy here.
    let kibibytes: Option<usize> = args.next().unwrap().parse().ok();

    // If "Sequential", we'll advise. Otherwise, not.
    // Very lazy here.
    let advise = args.next().unwrap() == "Sequential";

    let source = File::open(source_path).unwrap();
    let source = unsafe { Mmap::map(&source).unwrap() };

    if advise {
        source.advise(Advice::Sequential).unwrap();
    }

    let mut dest = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
        .unwrap();

    match kibibytes {
        Some(kib) => {
            for chunk in source.chunks(kib * 1024) {
                dest.write_all(chunk).unwrap();
            }
        }
        None => dest.write_all(&source).unwrap(),
    }
}
