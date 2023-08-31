use convert_case::{Case, Casing};
use sailfish::TemplateOnce;
use std::fs::{create_dir_all, read_dir};
use std::path::Path;
use std::process::exit;

use crate::frontend::templates;
use crate::util::{error, info, unwrap_or_exit, write_to_file};

static COMPONENT_DIRECTORY: &'static str = "src/components";
static COMPONENT_MOD: &'static str = "src/components/mod.rs";

pub struct Component {
    name: String,
}

impl Component {
    pub fn new(name: &str) -> Component {
        Component {
            name: name.to_string(),
        }
    }

    pub fn path(&self) -> String {
        self.name.to_case(Case::Snake).to_string()
    }

    pub fn name(&self) -> String {
        self.name.to_case(Case::UpperCamel).to_string()
    }

    pub fn full_path(&self) -> String {
        Path::new(COMPONENT_DIRECTORY)
            .join(&self.path())
            .display()
            .to_string()
    }

    pub fn controller_name(&self) -> String {
        self.path().replace("_", "-")
    }

    #[allow(dead_code)]
    pub fn controller_path(&self) -> String {
        format!("{}_controller.js", self.path())
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
        assert!(path.is_dir());

        let components = path.components();
        let name = components
            .clone()
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        Component::new(name)
    }
}

/// Add a new component.
pub fn add(name: &str, overwrite: bool) {
    let component = Component::new(name);
    let path = Path::new(COMPONENT_DIRECTORY).join(component.path());

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

    let stimulus_path = path.join(&format!("{}_controller.js", component.path()));
    unwrap_or_exit!(write_to_file(&stimulus_path, &stimulus));
    info(&format!("written {}", stimulus_path.display()));

    let rust_path = path.join("mod.rs");
    unwrap_or_exit!(write_to_file(&rust_path, &rust));
    info(&format!("written {}", rust_path.display()));

    let scss_path = path.join(&format!("{}.scss", component.path()));
    unwrap_or_exit!(write_to_file(&scss_path, &scss));
    info(&format!("written {}", scss_path.display()));

    update_modules();
}

/// Update `mod.rs` with all the components in `src/components`.
pub fn update_modules() {
    let mut modules = Vec::new();
    let mut paths: Vec<_> = unwrap_or_exit!(read_dir(COMPONENT_DIRECTORY))
        .map(|p| p.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    for path in paths {
        let path = path.path();
        if path.is_file() {
            continue;
        }

        let component = Component::from(Path::new(&path));
        modules.push(component);
    }

    let modules = unwrap_or_exit!(templates::Mod { modules }.render_once());

    unwrap_or_exit!(write_to_file(&Path::new(COMPONENT_MOD), &modules));
    info(&format!("written {}", COMPONENT_MOD));
}
