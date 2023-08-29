//! A tool to assemble and bundle our frontend components.

use clap::Parser;
use convert_case::{Case, Casing};
use glob::glob;
use std::env::set_current_dir;
use std::fs::{read_to_string, remove_file, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Ignore this, cargo passes in the name of the command as the first arg.
    subcomand: String,

    /// Path to the project directory.
    #[arg(short, long)]
    project_path: String,
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.project_path);

    if !path.exists() {
        panic!("Project path '{}' does not exist", path.display());
    }

    set_current_dir(path).expect("failed to change paths");

    // Assemble SCSS.
    let scss = glob("src/templates/**/*.scss").expect("failed to glob scss files");

    let mut modules =
        File::create("static/css/modules.scss").expect("failed to create modules.scss");

    for stylesheet in scss {
        let stylesheet = stylesheet.expect("failed to glob stylesheet");
        let line = format!(r#"@import "../../{}";"#, stylesheet.display());

        writeln!(&mut modules, "{}", line).expect("failed to write line to modules.scss");
    }

    drop(modules);

    // Bundle SCSS.
    // Build Bootstrap
    let sass = Command::new("sass")
        .arg("static/css/bootstrap-theme.scss")
        .arg("static/css/style.css")
        .status()
        .expect("`npm exec sass` failed");

    if !sass.success() {
        panic!("Sass compilatio failed");
    }

    // Hash the bundle.
    let bundle = read_to_string("static/css/style.css").expect("failed to read bundle.css");
    let hash = format!("{:x}", md5::compute(bundle))
        .chars()
        .take(8)
        .collect::<String>();

    if !Command::new("cp")
        .arg("static/css/style.css")
        .arg(format!("static/css/style.{}.css", hash))
        .status()
        .expect("cp static/css/style.css failed to run")
        .success()
    {
        panic!("Bundling CSS failed");
    }

    let mut hash_file =
        File::create("static/css/.pgml-bundle").expect("failed to create .pgml-bundle");
    writeln!(&mut hash_file, "{}", hash).expect("failed to write hash to .pgml-bundle");
    drop(hash_file);

    // Assemble JavaScript.

    // Remove prebuilt files.
    for file in glob::glob("static/js/*.*.js").expect("failed to glob") {
        let _ = remove_file(file.expect("failed to glob file"));
    }

    let js = glob("src/templates/**/*.js").expect("failed to glob js files");
    let js = js.chain(glob("static/js/*.js").expect("failed to glob static/js/*.js"));
    let js = js.filter(|path| {
        let path = path.as_ref().unwrap();
        let path = path.display().to_string();

        !path.contains("main.js") && !path.contains("bundle.js") && !path.contains("modules.js")
    });

    let mut modules = File::create("static/js/modules.js").expect("failed to create modules.js");

    writeln!(&mut modules, "// Build with --bin components").unwrap();
    writeln!(
        &mut modules,
        "import {{ Application }} from '@hotwired/stimulus'"
    )
    .expect("failed to write to modules.js");
    writeln!(&mut modules, "const application = Application.start()")
        .expect("failed to write to modules.js");

    for source in js {
        let source = source.expect("failed to glob js file");

        let full_path = source.display();
        let stem = source.file_stem().unwrap().to_str().unwrap();
        let upper_camel = stem.to_case(Case::UpperCamel);

        let mut controller_name = stem.split("_").collect::<Vec<&str>>();

        if stem.contains("controller") {
            let _ = controller_name.pop().unwrap();
        }

        let controller_name = controller_name.join("-");

        writeln!(
            &mut modules,
            "import {{ default as {} }} from '../../{}'",
            upper_camel, full_path
        )
        .unwrap();
        writeln!(
            &mut modules,
            "application.register('{}', {})",
            controller_name, upper_camel
        )
        .unwrap();
    }

    drop(modules);

    // Bundle JavaScript.
    let rollup = Command::new("rollup")
        .arg("static/js/modules.js")
        .arg("--file")
        .arg("static/js/bundle.js")
        .arg("--format")
        .arg("es")
        .status()
        .expect("`rollup` failed");

    if !rollup.success() {
        panic!("Rollup failed");
    }

    // Hash the bundle.
    let bundle = read_to_string("static/js/bundle.js").expect("failed to read bundle.js");
    let hash = format!("{:x}", md5::compute(bundle))
        .chars()
        .take(8)
        .collect::<String>();

    if !Command::new("cp")
        .arg("static/js/bundle.js")
        .arg(format!("static/js/bundle.{}.js", hash))
        .status()
        .expect("cp static/js/bundle.js failed to run")
        .success()
    {
        panic!("Bundling JavaScript failed");
    }

    let mut hash_file =
        File::create("static/js/.pgml-bundle").expect("failed to create .pgml-bundle");
    writeln!(&mut hash_file, "{}", hash).expect("failed to write hash to .pgml-bundle");
    drop(hash_file);
}
