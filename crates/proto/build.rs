const PROTO_FILES: [&str; 1] = ["proto/fleetwatch/control/v1/control.proto"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = std::env::var_os("CARGO_FEATURE_CLIENT").is_some();
    let server = std::env::var_os("CARGO_FEATURE_SERVER").is_some();

    tonic_prost_build::configure()
        .build_client(client)
        .build_server(server)
        .compile_protos(&PROTO_FILES, &["proto"])?;

    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
