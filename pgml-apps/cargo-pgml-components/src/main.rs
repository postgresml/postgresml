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
    // let component_name = name.as_str().to_case(Case::UpperCamel);
    // let component_path = name.as_str().to_case(Case::Snake);
    // let folder = Path::new("src/components").join(&component_path);

    // if !folder.exists() {
    //     match create_dir_all(folder.clone()) {
    //         Ok(_) => (),
    //         Err(err) => {
    //             error!(
    //                 "Failed to create path '{}' for component '{}': {}",
    //                 folder.display(),
    //                 name,
    //                 err
    //             );
    //             exit(1);
    //         }
    //     }
    // } else if !overwrite {
    //     error!("Component '{}' already exists", folder.display());
    //     exit(1);
    // }

    // // Create mod.rs
    // let mod_file = format!(
    //     "{}",
    //     COMPONENT_TEMPLATE_RS
    //         .replace("{component_name}", &component_name)
    //         .replace("{component_path}", &component_path)
    // );

    // let mod_path = folder.join("mod.rs");

    // let mut mod_file_fd = File::create(mod_path).expect("failed to create mod.rs");
    // writeln!(&mut mod_file_fd, "{}", mod_file.trim()).expect("failed to write mod.rs");
    // drop(mod_file_fd);

    // // Create template.html
    // let template_path = folder.join("template.html");
    // let mut template_file = File::create(template_path).expect("failed to create template.html");
    // let template_source =
    //     COMPONENT_HTML.replace("{controller_name}", &component_path.replace("_", "-"));
    // writeln!(&mut template_file, "{}", template_source.trim(),)
    //     .expect("failed to write template.html");
    // drop(template_file);

    // // Create Stimulus controller
    // let stimulus_path = folder.join(&format!("{}_controller.js", component_path));
    // let mut template_file =
    //     File::create(stimulus_path).expect("failed to create stimulus controller");
    // let controller_source =
    //     COMPONENT_STIMULUS_JS.replace("{controller_name}", &component_path.replace("_", "-"));
    // writeln!(&mut template_file, "{}", controller_source.trim())
    //     .expect("failed to write stimulus controller");
    // drop(template_file);

    // // Create SASS file
    // let sass_path = folder.join(&format!("{}.scss", component_path));
    // let sass_file = File::create(sass_path).expect("failed to create sass file");
    // drop(sass_file);

    // println!("Component '{}' created successfully", folder.display());
    // update_components();
}

fn update_components() {
    crate::frontend::components::update_modules();
    // let mut file = File::create("src/components/mod.rs").expect("failed to create mod.rs");

    // writeln!(
    //     &mut file,
    //     "// This file is automatically generated by cargo-pgml-components."
    // )
    // .expect("failed to write to mod.rs");
    // writeln!(&mut file, "// Do not modify it directly.").expect("failed to write to mod.rs");
    // writeln!(&mut file, "mod component;").expect("failed to write to mod.rs");
    // writeln!(
    //     &mut file,
    //     "pub(crate) use component::{{component, Component}};"
    // )
    // .expect("failed to write to mod.rs");

    // for component in read_dir("src/components").expect("failed to read components directory") {
    //     let path = component.expect("dir entry").path();

    //     if path.is_file() {
    //         continue;
    //     }

    //     let components = path.components();
    //     let component_name = components
    //         .clone()
    //         .last()
    //         .expect("component_name")
    //         .as_os_str()
    //         .to_str()
    //         .unwrap();
    //     let module = components
    //         .skip(2)
    //         .map(|c| c.as_os_str().to_str().unwrap())
    //         .collect::<Vec<&str>>()
    //         .join("::");
    //     // let module = format!("crate::{}", module);
    //     let component_name = component_name.to_case(Case::UpperCamel);

    //     writeln!(&mut file, "pub mod {};", module).expect("failed to write to mod.rs");
    //     writeln!(&mut file, "pub use {}::{};", module, component_name)
    //         .expect("failed to write to mod.rs");
    // }
}
