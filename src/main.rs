fn main() {
    let path = std::env::args().next_back().expect("Provide a path");
    let file = std::fs::File::open(path).expect("path should be readable");
    println!(
        "{}",
        file.metadata().expect("metadata should be gettable").len()
    );
}
