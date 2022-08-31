extern crate bindgen;
extern crate cmake;

use cmake::Config;
use std::process::Command;
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let xgb_root = Path::new(&out_dir).join("xgboost");

    // copy source code into OUT_DIR for compilation if it doesn't exist
    if !xgb_root.exists() {
        Command::new("cp")
            .args(&["-r", "xgboost", xgb_root.to_str().unwrap()])
            .status()
            .unwrap_or_else(|e| {
                panic!("Failed to copy ./xgboost to {}: {}", xgb_root.display(), e);
            });
    }

    // CMake
    let dst = Config::new(&xgb_root)
        .uses_cxx11()
        .define("BUILD_STATIC_LIB", "ON")
        .build();

    let xgb_root = xgb_root.canonicalize().unwrap();

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(&["-x", "c++", "-std=c++11"])
        .clang_arg(format!("-I{}", xgb_root.join("include").display()))
        .clang_arg(format!("-I{}", xgb_root.join("rabit/include").display()))
        .clang_arg(format!("-I{}", xgb_root.join("dmlc-core/include").display()))
        .generate()
        .expect("Unable to generate bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings.");

    println!("cargo:rustc-link-search={}", xgb_root.join("lib").display());
    println!("cargo:rustc-link-search={}", xgb_root.join("rabit/lib").display());
    println!("cargo:rustc-link-search={}", xgb_root.join("dmlc-core").display());

    // link to appropriate C++ lib
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=dylib=omp");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=dylib=gomp");
    }

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    println!("cargo:rustc-link-lib=static=dmlc");
    println!("cargo:rustc-link-lib=static=xgboost");
}
