//! A tool to assemble and bundle our frontend components.

use clap::{Args, Parser, Subcommand};
use convert_case::{Case, Casing};
use glob::glob;
use std::env::{current_dir, set_current_dir};
use std::fs::{create_dir_all, read_dir, read_to_string, remove_file, File};
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};

#[macro_use]
extern crate log;

mod frontend;
mod util;
use util::{execute_command, unwrap_or_exit};

/// These paths are exepcted to exist in the project directory.
static PROJECT_PATHS: &[&str] = &["src", "static/js", "static/css"];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, propagate_version = true, bin_name = "cargo", name = "cargo")]
struct Cli {
    #[command(subcommand)]
    subcomand: CargoSubcommands,
}

#[derive(Subcommand, Debug)]
enum CargoSubcommands {
    PgmlComponents(PgmlCommands),
}

#[derive(Args, Debug)]
struct PgmlCommands {
    #[command(subcommand)]
    command: Commands,

    /// Specify project path (default: current directory)
    #[arg(short, long)]
    project_path: Option<String>,

    /// Overwrite existing files (default: false)
    #[arg(short, long, default_value = "false")]
    overwrite: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Bundle SASS and JavaScript into neat bundle files.
    Bundle {},

    /// Add new elements to the project.
    #[command(subcommand)]
    Add(AddCommands),

    UpdateComponents {},
}

#[derive(Subcommand, Debug)]
enum AddCommands {
    /// Add a new component.
    Component { name: String },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.subcomand {
        CargoSubcommands::PgmlComponents(pgml_commands) => match pgml_commands.command {
            Commands::Bundle {} => bundle(pgml_commands.project_path),
            Commands::Add(command) => match command {
                AddCommands::Component { name } => add_component(name, pgml_commands.overwrite),
            },
            Commands::UpdateComponents {} => update_components(),
            _ => (),
        },
    }
}

/// Bundle SASS and JavaScript into neat bundle files.
fn bundle(project_path: Option<String>) {
    // Validate that the required project paths exist.
    let cwd = if let Some(project_path) = project_path {
        project_path
    } else {
        current_dir().unwrap().to_str().unwrap().to_owned()
    };

    let path = Path::new(&cwd);

    for project_path in PROJECT_PATHS {
        let check = path.join(project_path);

        if !check.exists() {
            unwrap_or_exit!(create_dir_all(check));
        }
    }

    set_current_dir(path).expect("failed to change paths");
    frontend::sass::bundle();

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

fn add_component(name: String, overwrite: bool) {
    crate::frontend::components::add(&name, overwrite);
}

fn update_components() {
    crate::frontend::components::update_modules();
}
