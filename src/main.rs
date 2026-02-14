fn main() {
    let path = std::env::args().next_back().expect("Provide a path");
    println!("{path:?}");
}
