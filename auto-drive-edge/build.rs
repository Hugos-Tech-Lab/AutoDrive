fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = std::path::PathBuf::from("external/protoc-36.2-win64/bin/protoc.exe");

    if !protoc.exists() {
        return Err(format!("protoc not found at: {}", protoc.display()).into());
    }

    // PROTOC tells prost-build where the compiler is.
    println!("cargo:rustc-env=PROTOC={}", protoc.display());
    unsafe { std::env::set_var("PROTOC", &protoc) };
    let proto_file = "api/auto_drive_edge_service/v1/auto_drive_edge.proto";

    println!("cargo:rerun-if-changed={proto_file}");

    tonic_prost_build::configure()
        .compile_protos(&[proto_file], &["api"])?;

    Ok(())
}