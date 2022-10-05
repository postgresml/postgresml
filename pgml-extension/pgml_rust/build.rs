fn main() {
    println!("cargo:rustc-link-search={}", "/opt/homebrew/lib/");
    println!("cargo:rustc-link-lib=static=openblas");
}
