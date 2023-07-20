fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=/opt/homebrew/opt/openblas/lib");
        println!("cargo:rustc-link-search=/opt/homebrew/opt/libomp/lib/");
    }
}
