use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the proto version from environment or use default.
    let proto_version = env::var("PROTO_VERSION").unwrap_or_else(|_| "v1".to_string());

    println!("cargo:rerun-if-changed=proto/plugin.proto");
    println!("cargo:rerun-if-env-changed=PROTO_VERSION");
    println!("cargo:rerun-if-env-changed=FORCE_CODEGEN");

    let proto_path = PathBuf::from("proto/plugin.proto");
    let generated_file = PathBuf::from("src/generated/mozilla.mcpd.plugins.v1.rs");

    // Check if we need to regenerate code.
    // Skip generation if both proto and generated files exist, unless FORCE_CODEGEN is set.
    let force_codegen = env::var("FORCE_CODEGEN").is_ok();
    let needs_generation = force_codegen || !proto_path.exists() || !generated_file.exists();

    if !needs_generation {
        eprintln!("Using existing generated code (set FORCE_CODEGEN=1 to regenerate)");
        return Ok(());
    }

    // Download proto file if it doesn't exist.
    if !proto_path.exists() {
        eprintln!("Downloading plugin.proto version {}...", proto_version);
        std::fs::create_dir_all("proto")?;

        let url = format!(
            "https://raw.githubusercontent.com/mozilla-ai/mcpd-proto/refs/heads/main/plugins/{}/plugin.proto",
            proto_version
        );

        // Use a simple HTTP client to download the file.
        let response = ureq::get(&url).call()?;
        let mut file = std::fs::File::create(&proto_path)?;
        std::io::copy(&mut response.into_reader(), &mut file)?;

        eprintln!("Downloaded proto file successfully");
    }

    // Ensure output directory exists.
    let out_dir = PathBuf::from("src/generated");
    std::fs::create_dir_all(&out_dir)?;

    // Configure protobuf compilation.
    eprintln!("Generating Rust code from protobuf...");
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir(&out_dir)
        .compile_protos(&["proto/plugin.proto"], &["proto"])?;

    Ok(())
}
