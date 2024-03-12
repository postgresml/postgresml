use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "profile_icon/template.html")]
pub struct ProfileIcon {
    pub profile_picture: Option<String>,
}

impl ProfileIcon {
    pub fn new(profile_picture: Option<String>) -> ProfileIcon {
        ProfileIcon { profile_picture }
    }
}

component!(ProfileIcon);
