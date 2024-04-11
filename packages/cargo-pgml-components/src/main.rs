//! A tool to assemble and bundle our frontend components.

use clap::{Args, Parser, Subcommand};
use file_lock::{FileLock, FileOptions};
use std::env::{current_dir, set_current_dir};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[macro_use]
extern crate log;

mod backend;
mod config;
mod frontend;
mod local_dev;
mod util;

use config::Config;
use util::{info, unwrap_or_exit};

/// These paths are exepcted to exist in the project directory.
static PROJECT_PATHS: &[&str] = &[
    "src",
    "static/js",
    "static/css",
    "templates/components",
    "src/components",
];

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
    Bundle {
        #[arg(short, long, default_value = "false")]
        minify: bool,

        #[arg(short, long, default_value = "false")]
        debug: bool,

        #[arg(short, long, default_value = "false")]
        lock: bool,
    },

    /// Add new elements to the project.
    #[command(subcommand)]
    Add(AddCommands),

    /// Setup local dev.
    #[command(subcommand)]
    LocalDev(LocalDevCommands),

    /// Watch for local changes
    Watch,

    /// Lint your code
    Lint {
        #[arg(long, default_value = "false")]
        check: bool,
    },
}

#[derive(Subcommand, Debug)]
enum AddCommands {
    /// Add a new component.
    Component {
        /// Name of the new component.
        name: String,

        /// Generate only the HTML template. Don't generate SCSS and JavaScript.
        #[arg(short, long, default_value = "false")]
        template_only: bool,
    },
}

#[derive(Subcommand, Debug)]
enum LocalDevCommands {
    /// Setup local dev.
    Check {},
    InstallPgvector {},
}

fn main() {
    let config = Config::load();
    env_logger::init();
    let cli = Cli::parse();

    match cli.subcomand {
        CargoSubcommands::PgmlComponents(pgml_commands) => {
            validate_project(pgml_commands.project_path);
            match pgml_commands.command {
                Commands::Bundle {
                    minify,
                    debug,
                    lock,
                } => bundle(config, minify, debug, lock),
                Commands::Add(command) => match command {
                    AddCommands::Component {
                        name,
                        template_only,
                    } => crate::frontend::components::add(
                        &Path::new(&name),
                        pgml_commands.overwrite,
                        template_only,
                    ),
                },
                Commands::LocalDev(command) => match command {
                    LocalDevCommands::Check {} => local_dev::setup(),
                    LocalDevCommands::InstallPgvector {} => local_dev::install_pgvector(),
                },
                Commands::Watch => {
                    frontend::tools::watch();
                }
                Commands::Lint { check } => {
                    frontend::tools::lint(check);
                }
            }
        }
    }
}

fn validate_project(project_path: Option<String>) {
    debug!("validating project directory");

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
            unwrap_or_exit!(create_dir_all(&check));
            info(&format!("created {} directory", check.display()));
        }
    }

    unwrap_or_exit!(set_current_dir(path));
}

/// Bundle SASS and JavaScript into neat bundle files.
fn bundle(config: Config, minify: bool, debug: bool, lock: bool) {
    let lock = if lock { Some(acquire_lock()) } else { None };

    if debug {
        frontend::tools::debug();
    }

    frontend::tools::install();

    if debug {
        frontend::tools::debug();
    }

    frontend::sass::bundle();
    frontend::javascript::bundle(config, minify);
    frontend::components::update_modules();

    info("bundle complete");

    if let Some(lock) = lock {
        unwrap_or_exit!(lock.unlock());
    }
}

fn acquire_lock() -> FileLock {
    print!("acquiring lock...");
    unwrap_or_exit!(std::io::stdout().flush());

    let file = "/tmp/pgml-components-lock";

    // Create file if not exists
    if !Path::new(file).exists() {
        unwrap_or_exit!(File::create(file));
    }

    let options = FileOptions::new().write(true).create(true).append(true);

    let lock = unwrap_or_exit!(FileLock::lock(file, true, options));

    info("ok");
    lock
}
