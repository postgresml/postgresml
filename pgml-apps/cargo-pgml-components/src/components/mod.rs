use crate::util::{compare_strings, info, unwrap_or_exit, write_to_file};
use std::fs::read_to_string;
use std::path::Path;

static COMPONENT_PATH: &str = "src/components/component.rs";
static COMPONENT_TEMPLATE: &str = "templates/components/component.html";

pub mod component;

pub fn install() {
    let source = include_str!("component.rs");
    let template = include_str!("component.html");

    compare_and_install(Path::new(COMPONENT_PATH), source);
    compare_and_install(Path::new(COMPONENT_TEMPLATE), template);
}

fn compare_and_install(path: &Path, source: &str) {
    if !path.exists() {
        debug!("{} doesn't exist", path.display());
        info(&format!("written {}", path.display()));
        unwrap_or_exit!(write_to_file(&path, &source));
    } else {
        let template_source = unwrap_or_exit!(read_to_string(path));

        if !compare_strings(&template_source, source) {
            debug!("{} is different", path.display());
            unwrap_or_exit!(write_to_file(&path, &source));
            info(&format!("written {}", path.display()));
        }
    }
}
