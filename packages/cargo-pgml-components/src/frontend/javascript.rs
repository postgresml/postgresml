//! Javascript bundling.

use glob::glob;
use std::collections::{HashMap, HashSet};
use std::fs::{copy, read_to_string, remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::frontend::tools::execute_with_nvm;
use crate::util::{error, info, unwrap_or_exit, warn};

/// The name of the JS file that imports all other JS files
/// created in the modules.
static MODULES_FILE: &'static str = "static/js/modules.js";

/// The JS bundle.
static JS_FILE: &'static str = "static/js/bundle.js";
static JS_FILE_HASHED: &'static str = "static/js/bundle.{}.js";
static JS_HASH_FILE: &'static str = "static/js/.pgml-bundle";

/// Finds all the JS files we have generated or the user has created.
static MODULES_GLOB: &'static str = "src/components/**/*.js";
static STATIC_JS_GLOB: &'static str = "static/js/*.js";

/// Finds old JS bundles we created.
static OLD_BUNLDES_GLOB: &'static str = "static/js/*.*.js";

/// JS compiler
static JS_COMPILER: &'static str = "rollup";

#[derive(Serialize, Deserialize, Debug)]
struct Packages {
    dependencies: HashMap<String, String>,
}

/// Delete old bundles we may have created.
fn cleanup_old_bundles() {
    // Clean up old bundles
    for file in unwrap_or_exit!(glob(OLD_BUNLDES_GLOB)) {
        let file = unwrap_or_exit!(file);
        debug!("removing {}", file.display());
        unwrap_or_exit!(remove_file(file.clone()));
        warn(&format!("deleted {}", file.display()));
    }
}

fn assemble_modules(config: Config) {
    let js = unwrap_or_exit!(glob(MODULES_GLOB));
    let mut js = js
        .chain(unwrap_or_exit!(glob(STATIC_JS_GLOB)))
        .collect::<Vec<_>>();

    for path in &config.javascript.additional_paths {
        debug!("adding additional path to javascript bundle: {}", path);
        js = js
            .into_iter()
            .chain(unwrap_or_exit!(glob(path)))
            .collect::<Vec<_>>();
    }

    // Don't bundle artifacts we produce.
    let js = js.iter().filter(|path| {
        let path = path.as_ref().unwrap();
        let path = path.display().to_string();

        !path.contains("main.") && !path.contains("bundle.") && !path.contains("modules.")
    });

    let mut modules = unwrap_or_exit!(File::create(MODULES_FILE));

    unwrap_or_exit!(writeln!(&mut modules, "// Build with --bin components"));
    unwrap_or_exit!(writeln!(
        &mut modules,
        "import {{ Application }} from '@hotwired/stimulus'"
    ));
    unwrap_or_exit!(writeln!(
        &mut modules,
        "const application = Application.start()"
    ));

    let mut dup_check = HashSet::new();

    // You can have controllers in static/js
    // or in their respective components folders.
    for source in js {
        let source = unwrap_or_exit!(source);

        let full_path = source.display().to_string();

        let path = source.components().collect::<Vec<_>>();

        assert!(!path.is_empty());

        let path = path.iter().collect::<PathBuf>();
        let components = path.components();
        let file_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let controller_name = if file_stem.ends_with("controller") {
            let mut parts = vec![];

            let pp = components
                .map(|c| c.as_os_str().to_str().expect("component to be valid utf-8"))
                .filter(|c| !c.ends_with(".js"))
                .collect::<Vec<&str>>();
            let mut saw_src = false;
            let mut saw_components = false;
            for p in pp {
                if p == "src" {
                    saw_src = true;
                } else if p == "components" {
                    saw_components = true;
                } else if saw_src && saw_components {
                    parts.push(p);
                }
            }

            assert!(!parts.is_empty());

            parts.join("_")
        } else {
            file_stem
        };
        let upper_camel = controller_name.to_case(Case::UpperCamel).to_string();
        let controller_name = controller_name.replace("_", "-");

        if !dup_check.insert(controller_name.clone()) {
            error(&format!("duplicate controller name: {}", controller_name));
            exit(1);
        }

        unwrap_or_exit!(writeln!(
            &mut modules,
            "import {{ default as {} }} from '../../{}'",
            upper_camel, full_path
        ));

        unwrap_or_exit!(writeln!(
            &mut modules,
            "application.register('{}', {})",
            controller_name, upper_camel
        ));
    }

    info(&format!("written {}", MODULES_FILE));
}

pub fn bundle(config: Config, minify: bool) {
    cleanup_old_bundles();
    assemble_modules(config.clone());

    let package_json = Path::new("package.json");

    let packages: Packages = if package_json.is_file() {
        let packages = unwrap_or_exit!(read_to_string(package_json));
        unwrap_or_exit!(serde_json::from_str(&packages))
    } else {
        warn("package.json not found, can't validate rollup output");
        serde_json::from_str(r#"{"dependencies": {}}"#).unwrap()
    };

    let mut command = Command::new(JS_COMPILER);

    command
        .arg(MODULES_FILE)
        .arg("--file")
        .arg(JS_FILE)
        .arg("--format")
        .arg("es")
        .arg("-p")
        .arg("@rollup/plugin-node-resolve");

    if minify {
        command.arg("-p").arg("@rollup/plugin-terser");
    }

    // Bundle JavaScript.
    info("bundling javascript with rollup");
    let output = unwrap_or_exit!(execute_with_nvm(&mut command));

    let lines = output.split("\n");
    for line in lines {
        for (package, _version) in &packages.dependencies {
            if line.contains(package) {
                error(&format!("unresolved import: {}", package));
                exit(1);
            }
        }
    }

    info(&format!("written {}", JS_FILE));

    // Hash the bundle.
    let bundle = unwrap_or_exit!(read_to_string(JS_FILE));
    let hash = format!("{:x}", md5::compute(bundle))
        .chars()
        .take(8)
        .collect::<String>();

    unwrap_or_exit!(copy(JS_FILE, &JS_FILE_HASHED.replace("{}", &hash)));
    info(&format!("written {}", JS_FILE_HASHED.replace("{}", &hash)));

    // Legacy, remove code from main.js into respective modules.
    unwrap_or_exit!(copy(
        "static/js/main.js",
        &format!("static/js/main.{}.js", &hash)
    ));
    info(&format!(
        "written {}",
        format!("static/js/main.{}.js", &hash)
    ));

    let mut hash_file = unwrap_or_exit!(File::create(JS_HASH_FILE));
    unwrap_or_exit!(writeln!(&mut hash_file, "{}", hash));

    info(&format!("written {}", JS_HASH_FILE));
}
