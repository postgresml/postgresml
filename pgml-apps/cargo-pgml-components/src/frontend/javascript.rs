//! Javascript bundling.

use glob::glob;
use std::fs::{copy, read_to_string, remove_file, File};
use std::io::Write;
use std::process::Command;

use convert_case::{Case, Casing};

use crate::frontend::tools::execute_with_nvm;
use crate::util::{info, unwrap_or_exit, warn};

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

fn assemble_modules() {
    let js = unwrap_or_exit!(glob(MODULES_GLOB));
    let js = js.chain(unwrap_or_exit!(glob(STATIC_JS_GLOB)));

    // Don't bundle artifacts we produce.
    let js = js.filter(|path| {
        let path = path.as_ref().unwrap();
        let path = path.display().to_string();

        !path.contains("main.js") && !path.contains("bundle.js") && !path.contains("modules.js")
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

    for source in js {
        let source = unwrap_or_exit!(source);

        let full_path = source.display();
        let stem = source.file_stem().unwrap().to_str().unwrap();
        let upper_camel = stem.to_case(Case::UpperCamel);

        let mut controller_name = stem.split("_").collect::<Vec<&str>>();

        if stem.contains("controller") {
            let _ = controller_name.pop().unwrap();
        }

        let controller_name = controller_name.join("-");

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

pub fn bundle() {
    cleanup_old_bundles();
    assemble_modules();

    // Bundle JavaScript.
    unwrap_or_exit!(execute_with_nvm(
        Command::new(JS_COMPILER)
            .arg(MODULES_FILE)
            .arg("--file")
            .arg(JS_FILE)
            .arg("--format")
            .arg("es"),
    ));

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
