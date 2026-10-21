fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("PROTOC").is_err() {
        let protoc = std::path::PathBuf::from(
            "../auto-drive-edge/external/protoc-36.2-win64/bin/protoc.exe",
        );

        if !protoc.exists() {
            return Err(format!("protoc not found at: {}", protoc.display()).into());
        }

        println!("cargo:rustc-env=PROTOC={}", protoc.display());
        unsafe { std::env::set_var("PROTOC", &protoc) };
    }

    let proto_file = "../auto-drive-edge/api/auto_drive_edge_service/v1/auto_drive_edge.proto";

    println!("cargo:rerun-if-changed={proto_file}");

    tonic_prost_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&[proto_file], &["../auto-drive-edge/api"])?;

    Ok(())
}
