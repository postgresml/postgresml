use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if target.contains("apple") {
        println!("cargo:rustc-link-lib=static=openblas");
    }
}
