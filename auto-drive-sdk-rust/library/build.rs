use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let wit_file = out_dir.join("app.wit");

    println!("cargo:rustc-env=GENERATED_WIT={}", wit_file.display());
}
