//! A tool to assemble and bundle our frontend components.

use clap::{Args, Parser, Subcommand};
use convert_case::{Case, Casing};
use glob::glob;
use std::env::{current_dir, set_current_dir};
use std::fs::{create_dir_all, read_to_string, remove_file, File};
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};

#[macro_use]
extern crate log;

/// These paths are exepcted to exist in the project directory.
static PROJECT_PATHS: &[&str] = &["src", "static/js", "static/css"];

//// These executables are required to be installed globally.
static REQUIRED_EXECUTABLES: &[&str] = &["sass", "rollup"];

static COMPONENT_TEMPLATE_RS: &'static str = r#"
use sailfish::TemplateOnce;
use crate::components::component;

#[derive(TemplateOnce, Default)]
#[template(path = "{component_path}/template.html")]
pub struct {component_name} {
    value: String,
}

impl {component_name} {
    pub fn new() -> {component_name} {
        {component_name}::default()
    }
}

component!({component_name});
"#;

static COMPONENT_STIMULUS_JS: &'static str = r#"
import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = []
  static outlets = []

  initialize() {
    console.log('Initialized {controller_name}')
  }

  connect() {}

  disconnect() {}
}
"#;

static COMPONENT_HTML: &'static str = r#"
<div data-controller="{controller_name}">
  <%= value %>
</div>
"#;

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

    #[arg(short, long)]
    project_path: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Bundle SASS and JavaScript into neat bundle files.
    Bundle {},

    /// Add a new component.
    AddComponent {
        name: String,

        #[arg(short, long, default_value = "false")]
        overwrite: bool,
    },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.subcomand {
        CargoSubcommands::PgmlComponents(pgml_commands) => match pgml_commands.command {
            Commands::Bundle {} => bundle(pgml_commands.project_path),
            Commands::AddComponent { name, overwrite } => add_component(name, overwrite),
        },
    }
}

fn execute_command(command: &mut Command) -> std::io::Result<String> {
    let output = match command.output() {
        Ok(output) => output,
        Err(err) => {
            return Err(err);
        }
    };

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

fn check_executables() {
    for executable in REQUIRED_EXECUTABLES {
        match execute_command(Command::new(executable).arg("--version")) {
            Ok(_) => (),
            Err(err) => {
                error!(
                    "'{}' is not installed. Install it with 'npm install -g {}'",
                    executable, executable
                );
                debug!(
                    "Failed to execute '{} --version': {}",
                    executable,
                    err.to_string()
                );
                exit(1);
            }
        }
    }
}

/// Bundle SASS and JavaScript into neat bundle files.
fn bundle(project_path: Option<String>) {
    check_executables();

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

fn add_component(name: String, overwrite: bool) {
    let component_name = name.as_str().to_case(Case::UpperCamel);
    let component_path = name.as_str().to_case(Case::Snake);
    let folder = Path::new("src/components").join(&component_path);

    if !folder.exists() {
        match create_dir_all(folder.clone()) {
            Ok(_) => (),
            Err(err) => {
                error!(
                    "Failed to create path '{}' for component '{}': {}",
                    folder.display(),
                    name,
                    err
                );
                exit(1);
            }
        }
    } else if !overwrite {
        error!("Component '{}' already exists", folder.display());
        exit(1);
    }

    // Create mod.rs
    let mod_file = format!(
        "{}",
        COMPONENT_TEMPLATE_RS
            .replace("{component_name}", &component_name)
            .replace("{component_path}", &component_path)
    );

    let mod_path = folder.join("mod.rs");

    let mut mod_file_fd = File::create(mod_path).expect("failed to create mod.rs");
    writeln!(&mut mod_file_fd, "{}", mod_file.trim()).expect("failed to write mod.rs");
    drop(mod_file_fd);

    // Create template.html
    let template_path = folder.join("template.html");
    let mut template_file = File::create(template_path).expect("failed to create template.html");
    let template_source =
        COMPONENT_HTML.replace("{controller_name}", &component_path.replace("_", "-"));
    writeln!(&mut template_file, "{}", template_source.trim(),)
        .expect("failed to write template.html");
    drop(template_file);

    // Create Stimulus controller
    let stimulus_path = folder.join(&format!("{}_controller.js", component_path));
    let mut template_file =
        File::create(stimulus_path).expect("failed to create stimulus controller");
    let controller_source =
        COMPONENT_STIMULUS_JS.replace("{controller_name}", &component_path.replace("_", "-"));
    writeln!(&mut template_file, "{}", controller_source.trim())
        .expect("failed to write stimulus controller");
    drop(template_file);

    // let mut components_list = File::create("src/components/components.rs").expect("failed to create src/components/components.rs");
    // let components = read_dir("src/components").expect("failed to read components directory");

    println!("Component '{}' created successfully", folder.display());
    println!("Don't forget to add it to src/components/mod.rs");
}
