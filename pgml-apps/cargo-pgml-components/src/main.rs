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
use util::{execute_command, info, unwrap_or_exit};

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

    unwrap_or_exit!(set_current_dir(path));
    frontend::sass::bundle();
    frontend::javascript::bundle();

    info("Bundle complete");
}

fn add_component(name: String, overwrite: bool) {
    crate::frontend::components::add(&name, overwrite);
}

fn update_components() {
    crate::frontend::components::update_modules();
}
