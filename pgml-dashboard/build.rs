use std::fs::{read_to_string};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=migrations");

    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_SHA={}", git_hash);

    let css_version = read_to_string("static/css/.pgml-bundle")
        .expect("failed to read .pgml-bundle");
    let css_version = css_version.trim();

    let js_version = read_to_string("static/js/.pgml-bundle")
        .expect("failed to read .pgml-bundle");
    let js_version = js_version.trim();

    println!("cargo:rustc-env=CSS_VERSION={css_version}");
    println!("cargo:rustc-env=JS_VERSION={js_version}");
}
