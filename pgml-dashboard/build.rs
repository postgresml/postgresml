use std::fs::{read_to_string, remove_file};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=migrations");

    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_SHA={}", git_hash);

    // Build Bootstrap
    let status = Command::new("npm")
        .arg("exec")
        .arg("sass")
        .arg("static/css/bootstrap-theme.scss")
        .arg("static/css/style.css")
        .status()
        .unwrap();

    if !status.success() {
        println!("SCSS compilation failed");
    }

    // Bundle CSS to bust cache.
    let contents = read_to_string("static/css/style.css")
        .unwrap()
        .as_bytes()
        .to_vec();
    let css_version = format!("{:x}", md5::compute(contents))
        .chars()
        .take(8)
        .collect::<String>();

    if !Command::new("cp")
        .arg("static/css/style.css")
        .arg(format!("static/css/style.{}.css", css_version))
        .status()
        .unwrap()
        .success()
    {
        println!("Bundling CSS failed");
    }

    let mut js_version = Vec::new();

    // Remove all bundled files
    for file in glob::glob("static/js/*.*.js").expect("failed to glob") {
        let _ = remove_file(file.expect("failed to glob file"));
    }

    // Build JS to bust cache
    for file in glob::glob("static/js/*.js").expect("failed to glob") {
        let file = file.expect("failed to glob path");
        let contents = read_to_string(file).unwrap().as_bytes().to_vec();

        js_version.push(format!("{:x}", md5::compute(contents)));
    }

    let js_version = format!("{:x}", md5::compute(js_version.join("").as_bytes()))
        .chars()
        .take(8)
        .collect::<String>();

    for file in glob::glob("static/js/*.js").expect("failed to glob JS") {
        let filename = file.expect("failed to glob path").display().to_string();
        let name = filename.split(".").collect::<Vec<&str>>();
        let name = name[0..name.len() - 1].join(".");

        if !Command::new("cp")
            .arg(&filename)
            .arg(format!("{}.{}.js", name, js_version))
            .status()
            .unwrap()
            .success()
        {
            println!("Bundling JS failed");
        }
    }

    println!("cargo:rustc-env=CSS_VERSION={css_version}");
    println!("cargo:rustc-env=JS_VERSION={js_version}");
}
