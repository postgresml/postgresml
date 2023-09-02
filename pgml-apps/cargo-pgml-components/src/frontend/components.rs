use convert_case::{Case, Casing};
use sailfish::TemplateOnce;
use std::fs::{create_dir_all, read_dir, read_to_string};
use std::path::{Path, PathBuf};
use std::process::exit;

use crate::frontend::templates;
use crate::util::{compare_strings, error, info, unwrap_or_exit, write_to_file};

static COMPONENT_DIRECTORY: &'static str = "src/components";
static COMPONENT_MOD: &'static str = "src/components/mod.rs";

#[derive(Clone)]
pub struct Component {
    name: String,
    path: PathBuf,
    is_node: bool,
}

impl Component {
    pub fn new(name: &str, path: &Path) -> Component {
        Component {
            name: name.to_owned(),
            path: path.to_owned(),
            is_node: has_more_modules(path),
        }
    }

    pub fn path(&self) -> String {
        self.path.display().to_string()
    }

    pub fn name(&self) -> String {
        self.name.to_case(Case::Snake).to_string()
    }

    pub fn is_node(&self) -> bool {
        self.is_node
    }

    pub fn rust_name(&self) -> String {
        self.name.to_case(Case::UpperCamel).to_string()
    }

    pub fn full_path(&self) -> PathBuf {
        Path::new(COMPONENT_DIRECTORY)
            .join(&self.path)
            .to_owned()
    }

    pub fn controller_name(&self) -> String {
        self.name.to_case(Case::Snake).replace("_", "-")
    }

    pub fn controller_path(&self) -> String {
        format!("{}_controller.js", self.controller_name())
    }

    pub fn rust_module(&self) -> String {
        let full_path = self.full_path();
        let path = Path::new(&full_path);
        let components = path.components();

        components
            .skip(2) // skip src/components
            .map(|c| c.as_os_str().to_str().unwrap())
            .collect::<Vec<&str>>()
            .join("::")
            .to_string()
    }
}

impl From<&Path> for Component {
    fn from(path: &Path) -> Self {
        let components = path.components();
        let name = components
            .clone()
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        Component::new(name, path)
    }
}

/// Add a new component.
pub fn add(path: &Path, overwrite: bool) {
    if let Some(_extension) = path.extension() {
        error("component name should not contain an extension");
        exit(1);
    }

    let component = Component::from(path);
    let path = component.full_path();

    if path.exists() && !overwrite {
        error(&format!("component {} already exists", component.path()));
        exit(1);
    } else {
        unwrap_or_exit!(create_dir_all(&path));
        info(&format!("created directory {}", path.display()));
    }

    let rust = unwrap_or_exit!(templates::Component::new(&component).render_once());
    let stimulus = unwrap_or_exit!(templates::Stimulus::new(&component).render_once());
    let html = unwrap_or_exit!(templates::Html::new(&component).render_once());
    let scss = String::new();

    let html_path = path.join("template.html");
    unwrap_or_exit!(write_to_file(&html_path, &html));
    info(&format!("written {}", html_path.display()));

    let stimulus_path = path.join(&component.controller_path());
    unwrap_or_exit!(write_to_file(&stimulus_path, &stimulus));
    info(&format!("written {}", stimulus_path.display()));

    let rust_path = path.join("mod.rs");
    unwrap_or_exit!(write_to_file(&rust_path, &rust));
    info(&format!("written {}", rust_path.display()));

    let scss_path = path.join(&format!("{}.scss", component.name()));
    unwrap_or_exit!(write_to_file(&scss_path, &scss));
    info(&format!("written {}", scss_path.display()));

    update_modules();
}

/// Update `mod.rs` with all the components in `src/components`.
pub fn update_modules() {
    update_module(Path::new(COMPONENT_DIRECTORY), true);
    // let mut modules = Vec::new();
    // let mut paths: Vec<_> = unwrap_or_exit!(read_dir(COMPONENT_DIRECTORY))
    //     .map(|p| p.unwrap())
    //     .collect();
    // paths.sort_by_key(|dir| dir.path());

    // for path in paths {
    //     let path = path.path();
    //     if path.is_file() {
    //         continue;
    //     }

    //     let component = Component::from(Path::new(&path));
    //     modules.push(component);
    // }

    // let modules = unwrap_or_exit!(templates::Mod { modules, root: true }.render_once());
    // let existing_modules = unwrap_or_exit!(read_to_string(COMPONENT_MOD));

    // if !compare_strings(&modules, &existing_modules) {
    //     debug!("mod.rs is different");
    //     unwrap_or_exit!(write_to_file(&Path::new(COMPONENT_MOD), &modules));
    //     info(&format!("written {}", COMPONENT_MOD));
    // }

    // debug!("mod.rs is the same");
}

fn update_module(path: &Path, root: bool) {
    let mut modules = Vec::new();
    let mut paths: Vec<_> = unwrap_or_exit!(read_dir(path))
        .map(|p| p.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    for path in paths {
        let path = path.path();
        if path.is_file() {
            continue;
        }

        if has_more_modules(&path) {
            update_module(&path, false);
        }

        let component = Component::from(Path::new(&path));
        modules.push(component);
    }

    let components_mod = path.join("mod.rs");
    let modules = unwrap_or_exit!(templates::Mod { modules, root }.render_once());

    let existing_modules = if components_mod.is_file() {
        unwrap_or_exit!(read_to_string(&components_mod))
    } else {
        String::new()
    };

    if !compare_strings(&modules, &existing_modules) {
        debug!("{}/mod.rs is different", components_mod.display());
        unwrap_or_exit!(write_to_file(&components_mod, &modules));
        info(&format!("written {}", components_mod.display().to_string()));
    }

    debug!("mod.rs is the same");
}

fn has_more_modules(path: &Path) -> bool {
    assert!(path.is_dir());

    let paths = unwrap_or_exit!(read_dir(path));
    let paths = paths.map(|path| path.unwrap().path().to_owned());
    let files = paths.filter(|path| path.is_file()).filter(|path| path.file_name().unwrap() != "mod.rs").count();

    let only_has_mod = files == 0;

    debug!("{} has more modules: {}", path.display(), only_has_mod);

    only_has_mod
}
