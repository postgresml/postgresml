use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "profile_icon/template.html")]
pub struct ProfileIcon;

impl ProfileIcon {
    pub fn new() -> ProfileIcon {
        ProfileIcon
    }
}

component!(ProfileIcon);
