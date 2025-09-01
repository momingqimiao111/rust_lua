use std::{env, fs};
use std::path::Path;

// build.rs
fn main() {
    println!("cargo:warning=Build script is running");
    println!("cargo:warning=Setting up Lua library linking");
    println!("cargo:rustc-link-search=/dp2/lib");
    println!("cargo:rustc-link-lib=lua53");

    println!("cargo:rerun-if-changed=build.rs");

    let profile = env::var("PROFILE").unwrap();
    if profile == "release" {
        let target = env::var("TARGET").unwrap();
        let src_path = format!("target/{}/librust_websocket.so", target);
        let dst_path = format!("target/{}/rust_websocket.so", target);

        if Path::new(&src_path).exists() {
            println!("cargo:warning=Copying {} to {}", src_path, dst_path);
            let _ = fs::rename(&src_path, &dst_path);
        } else {
            println!("cargo:warning={} does not exist", src_path);
        }
    }
}
