// Custom build script to use pre-built llama.cpp libraries
use std::env;
use std::path::PathBuf;

/// Validates version consistency to prevent Issue #63 version mismatch problems
fn validate_version() {
    // Get version from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");

    // Validate version is not the default placeholder
    if version == "0.1.0" {
        panic!(
            "ERROR: Version is set to default 0.1.0\n\
             This suggests the package was not properly configured.\n\
             Please ensure Cargo.toml has the correct version number.\n\
             This prevents the version mismatch issue reported in Issue #63."
        );
    }

    // Validate version is not empty
    if version.is_empty() {
        panic!("ERROR: CARGO_PKG_VERSION is empty. Check your build environment.");
    }

    // Validate semantic versioning format
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 {
        panic!(
            "ERROR: Version '{}' does not follow semantic versioning (major.minor.patch)\n\
             Please use a valid version format like '1.4.2'",
            version
        );
    }

    // Validate each version component is numeric
    for (i, part) in parts.iter().take(3).enumerate() {
        if part.parse::<u32>().is_err() {
            panic!(
                "ERROR: Version component '{}' at position {} is not a valid number\n\
                 Version: {}",
                part, i, version
            );
        }
    }

    // Set build-time version for verification
    println!("cargo:rustc-env=SHIMMY_BUILD_VERSION={}", version);

    // Rebuild if version-related files change
    println!("cargo:rerun-if-changed=Cargo.toml");

    println!("cargo:warning=Building shimmy version {}", version);
}

fn main() {
    // Version validation - prevents Issue #63 version mismatch problems
    validate_version();

    println!("cargo:rerun-if-changed=libs/");

    // Check if we should use pre-built libraries
    if env::var("SHIMMY_USE_PREBUILT_LLAMA").is_ok() {
        println!("cargo:warning=Using pre-built llama.cpp libraries");

        let target = env::var("TARGET").unwrap();
        let lib_dir = match target.as_str() {
            "x86_64-pc-windows-msvc" => "libs/windows-x86_64",
            "x86_64-apple-darwin" => "libs/macos-intel",
            "aarch64-apple-darwin" => "libs/macos-arm64",
            "x86_64-unknown-linux-gnu" => "libs/linux-x86_64",
            "aarch64-unknown-linux-gnu" => "libs/linux-arm64",
            _ => {
                println!(
                    "cargo:warning=No pre-built library for target {}, falling back to compilation",
                    target
                );
                return;
            }
        };

        // Check if the library exists
        let lib_path = PathBuf::from(lib_dir).join(if target.contains("windows") {
            "llama.lib"
        } else {
            "libllama.a"
        });
        if lib_path.exists() {
            println!(
                "cargo:warning=Found pre-built library: {}",
                lib_path.display()
            );

            // Tell Cargo where to find the library
            println!("cargo:rustc-link-search=native={}", lib_dir);
            println!("cargo:rustc-link-lib=static=llama");

            // Set environment variables to tell llama-cpp-sys-2 to skip building
            println!("cargo:rustc-env=LLAMA_CPP_PREBUILT=1");
            println!("cargo:rustc-env=LLAMA_CPP_LIB_DIR={}", lib_dir);
            println!("cargo:rustc-env=LLAMA_CPP_SKIP_BUILD=1");
        } else {
            println!(
                "cargo:warning=Pre-built library not found: {}, falling back to compilation",
                lib_path.display()
            );
        }
    }
}
