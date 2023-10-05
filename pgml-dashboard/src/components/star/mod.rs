use std::collections::HashMap;

use pgml_components::component;
use once_cell::sync::Lazy;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "star/template.html")]
pub struct Star {
    content: String,
    id: Option<String>,
    svg: &'static str,
}

const SVGS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "green",
        include_str!("../../../static/images/icons/stars/green.svg"),
    );
    map.insert(
        "party",
        include_str!("../../../static/images/icons/stars/party.svg"),
    );
    map
});

impl Star {
    pub fn new<S: ToString, I: Into<Option<S>>>(color: &str, content: S, id: I) -> Star {
        Star {
            svg: SVGS.get(color).expect("Invalid star color"),
            content: content.to_string(),
            id: id.into().map(|s| s.to_string()),
        }
    }
}

component!(Star);
