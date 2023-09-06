use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "github_icon/template.html")]
pub struct GithubIcon {
    pub show_stars: bool,
}

impl GithubIcon {
    pub fn new() -> GithubIcon {
        GithubIcon::default()
    }
}

component!(GithubIcon);
