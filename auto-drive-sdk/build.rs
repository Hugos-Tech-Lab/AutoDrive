use std::fs;
use std::path::PathBuf;

use auto_drive_interface::the_beginner::WIT;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let wit_file = out_dir.join("app.wit");

    fs::write(&wit_file, WIT).unwrap();

    println!("cargo:rustc-env=GENERATED_WIT={}", wit_file.display());
}
