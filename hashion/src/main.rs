mod hasher;

use std::path::Path;

fn main() {
    let hasher = hasher::Hasher::new();
    let checksum = hasher.hash_file(Path::new("Cargo.toml")).unwrap();
    println!("{:x?}", checksum);
}
