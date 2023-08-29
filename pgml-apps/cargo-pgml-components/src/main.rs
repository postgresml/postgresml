//! A tool to assemble and bundle our frontend components.

use clap::Parser;
use convert_case::{Case, Casing};
use glob::glob;
use std::env::{current_dir, set_current_dir};
use std::fs::{read_to_string, remove_file, File};
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};

#[macro_use]
extern crate log;

/// These paths are exepcted to exist in the project directory.
static PROJECT_PATHS: &[&str] = &["src", "static/js", "static/css"];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Ignore this, cargo passes in the name of the command as the first arg.
    subcomand: String,

    /// Path to the project directory.
    #[arg(short, long)]
    project_path: Option<String>,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // Validate that the required project paths exist.
    let cwd = if let Some(project_path) = args.project_path {
        project_path
    } else {
        current_dir().unwrap().to_str().unwrap().to_owned()
    };

    let path = Path::new(&cwd);

    for project_path in PROJECT_PATHS {
        let check = path.join(project_path);

        if !check.exists() {
            error!(
                "Project path '{}/{}' does not exist but is required",
                path.display(),
                project_path
            );
            exit(1);
        }
    }

    set_current_dir(path).expect("failed to change paths");

    // Assemble SCSS.
    let scss = glob("src/components/**/*.scss").expect("failed to glob scss files");

    let mut modules =
        File::create("static/css/modules.scss").expect("failed to create modules.scss");

    for stylesheet in scss {
        let stylesheet = stylesheet.expect("failed to glob stylesheet");

        debug!("Adding '{}' to SCSS bundle", stylesheet.display());

        let line = format!(r#"@import "../../{}";"#, stylesheet.display());

        writeln!(&mut modules, "{}", line).expect("failed to write line to modules.scss");
    }

    drop(modules);

    // Bundle SCSS.
    // Build Bootstrap
    execute_command(
        Command::new("sass")
            .arg("static/css/bootstrap-theme.scss")
            .arg("static/css/style.css"),
    )
    .unwrap();

    // Hash the bundle.
    let bundle = read_to_string("static/css/style.css").expect("failed to read bundle.css");
    let hash = format!("{:x}", md5::compute(bundle))
        .chars()
        .take(8)
        .collect::<String>();

    execute_command(
        Command::new("cp")
            .arg("static/css/style.css")
            .arg(format!("static/css/style.{}.css", hash)),
    )
    .unwrap();

    let mut hash_file =
        File::create("static/css/.pgml-bundle").expect("failed to create .pgml-bundle");
    writeln!(&mut hash_file, "{}", hash).expect("failed to write hash to .pgml-bundle");
    drop(hash_file);

    debug!("Created css .pgml-bundle with hash {}", hash);

    // Assemble JavaScript.

    // Remove prebuilt files.
    for file in glob::glob("static/js/*.*.js").expect("failed to glob") {
        let _ = remove_file(file.expect("failed to glob file"));
    }

    let js = glob("src/components/**/*.js").expect("failed to glob js files");
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
    execute_command(
        Command::new("rollup")
            .arg("static/js/modules.js")
            .arg("--file")
            .arg("static/js/bundle.js")
            .arg("--format")
            .arg("es"),
    )
    .unwrap();

    // Hash the bundle.
    let bundle = read_to_string("static/js/bundle.js").expect("failed to read bundle.js");
    let hash = format!("{:x}", md5::compute(bundle))
        .chars()
        .take(8)
        .collect::<String>();

    execute_command(
        Command::new("cp")
            .arg("static/js/bundle.js")
            .arg(format!("static/js/bundle.{}.js", hash)),
    )
    .unwrap();

    let mut hash_file =
        File::create("static/js/.pgml-bundle").expect("failed to create .pgml-bundle");
    writeln!(&mut hash_file, "{}", hash).expect("failed to write hash to .pgml-bundle");
    drop(hash_file);

    println!("Finished bundling CSS and JavaScript successfully");
}

fn execute_command(command: &mut Command) -> std::io::Result<String> {
    let output = command.output()?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        error!(
            "{} failed: {}",
            command.get_program().to_str().unwrap(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        );
        exit(1);
    }

    if !stderr.is_empty() {
        warn!("{}", stderr);
    }

    if !stdout.is_empty() {
        info!("{}", stdout);
    }

    Ok(stdout)
}
