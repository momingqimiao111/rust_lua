// build.rs
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("执行脚本");
    println!("cargo:rerun-if-changed=src/lib.rs");

    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();

    println!("Build script running...");
    println!("Target: {}", target);
    println!("Profile: {}", profile);

    // 注意：debug 和 release 的路径结构不同
    let src_path = if profile == "release" {
        format!("target/{}/{}/librust_websocket.so", target, profile)
    } else {
        format!("target/{}/librust_websocket.so", target)
    };

    let dst_path = if profile == "release" {
        format!("target/{}/{}/rust_websocket.so", target, profile)
    } else {
        format!("target/{}/rust_websocket.so", target)
    };

    println!("Source path: {}", src_path);
    println!("Destination path: {}", dst_path);

    if Path::new(&src_path).exists() {
        match fs::rename(&src_path, &dst_path) {
            Ok(_) => println!("Successfully renamed file"),
            Err(e) => println!("Failed to rename file: {}", e),
        }
    } else {
        println!("Source file does not exist");
    }
}
