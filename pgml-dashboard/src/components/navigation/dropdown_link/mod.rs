use crate::components::StaticNavLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/dropdown_link/template.html")]
pub struct DropdownLink {
    link: StaticNavLink,
}

impl DropdownLink {
    pub fn new(link: StaticNavLink) -> Self {
        DropdownLink { link }
    }
}

component!(DropdownLink);
