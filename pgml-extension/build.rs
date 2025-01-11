fn main() {
    println!("cargo::rustc-check-cfg=cfg(pgrx_embed)");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=/opt/homebrew/opt/openblas/lib");
        println!("cargo:rustc-link-search=/opt/homebrew/opt/libomp/lib");
    }

    // PostgreSQL is using dlopen(RTLD_GLOBAL). This will parse some
    // of the symbols into the previous opened .so file, but the others will use a
    // relative offset in pgml.so, and will cause a null-pointer crash.
    //
    // hide all symbol to avoid symbol conflicts.
    //
    // append mode (link-args) only works with clang ld (lld)
    println!(
        "cargo:link-args=-Wl,--version-script={}/ld.map",
        std::env::current_dir().unwrap().to_string_lossy(),
    );

    vergen::EmitBuilder::builder().all_git().emit().unwrap();
}
