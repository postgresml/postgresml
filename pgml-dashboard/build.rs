use glob::glob;
use std::collections::BTreeSet;
use std::fs::read_to_string;
use std::hash::Hasher;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=migrations");

    let output = Command::new("git").args(["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_SHA={}", git_hash);

    for i in 0..5 {
        let status = Command::new("cargo")
            .arg("pgml-components")
            .arg("bundle")
            .arg("--lock")
            .status()
            .expect("failed to run 'cargo pgml-bundle'");

        if !status.success() {
            if i < 4 {
                println!("cargo:warning=failed to run 'cargo pgml-bundle', retrying");
            } else {
                panic!("failed to run 'cargo pgml-bundle'");
            }
        }
    }

    let css_version = read_to_string("static/css/.pgml-bundle").expect("failed to read .pgml-bundle");
    let css_version = css_version.trim();
    println!("cargo:rustc-env=CSS_VERSION={css_version}");

    let js_version = read_to_string("static/js/.pgml-bundle").expect("failed to read .pgml-bundle");
    let js_version = js_version.trim();
    println!("cargo:rustc-env=JS_VERSION={js_version}");

    let status = Command::new("cp")
        .arg("static/js/main.js")
        .arg(&format!("static/js/main.{}.js", js_version))
        .status()
        .expect("failed to bundle main.js");

    if !status.success() {
        panic!("failed to bundle main.js");
    }

    let files_paths = glob("./../pgml-cms/**/*.md")
        .expect("Failed to read pgml-cms directory")
        .map(|p| p.unwrap())
        .collect::<BTreeSet<PathBuf>>();
    let mut hasher = std::hash::DefaultHasher::new();
    for path in files_paths {
        let contents = read_to_string(path.clone()).expect("Error reading file");
        hasher.write(&contents.into_bytes());
    }
    let cms_hash = hasher.finish();
    println!("cargo:rustc-env=CMS_HASH={cms_hash}");
}
