use std::path::PathBuf;

fn main() {
    let wit = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("interface");

    if !wit.is_dir() {
        panic!("Firmware WIT directory does not exist: {}", wit.display());
    }

    println!("cargo::metadata=wit={}", wit.display());
    println!("cargo::rerun-if-changed={}", wit.display());
}
