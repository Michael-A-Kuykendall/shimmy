// Custom build script to use pre-built llama.cpp libraries
use std::env;
use std::path::PathBuf;

fn main() {
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
                println!("cargo:warning=No pre-built library for target {}, falling back to compilation", target);
                return;
            }
        };
        
        // Check if the library exists
        let lib_path = PathBuf::from(lib_dir).join(if target.contains("windows") { "llama.lib" } else { "libllama.a" });
        if lib_path.exists() {
            println!("cargo:warning=Found pre-built library: {}", lib_path.display());
            
            // Tell Cargo where to find the library
            println!("cargo:rustc-link-search=native={}", lib_dir);
            println!("cargo:rustc-link-lib=static=llama");
            
            // Set environment variables to tell llama-cpp-sys-2 to skip building
            println!("cargo:rustc-env=LLAMA_CPP_PREBUILT=1");
            println!("cargo:rustc-env=LLAMA_CPP_LIB_DIR={}", lib_dir);
            println!("cargo:rustc-env=LLAMA_CPP_SKIP_BUILD=1");
        } else {
            println!("cargo:warning=Pre-built library not found: {}, falling back to compilation", lib_path.display());
        }
    }
}