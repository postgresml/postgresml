use pgml_components::{component, Component};
use sailfish::TemplateOnce;

/// A component that renders a Bootstrap modal.
#[derive(TemplateOnce, Default)]
#[template(path = "modal/template.html")]
pub struct Modal {
    pub id: String,
    pub size_class: String,
    pub header: Option<Component>,
    pub body: Component,
    pub default_style: bool,
    static_backdrop: String,
}

component!(Modal);

impl Modal {
    /// Create a new x-large modal with the given body.
    pub fn new(body: Component) -> Self {
        let modal = Modal {
            default_style: true,
            ..Modal::default()
        };
        let id = format!("modal-{}", crate::utils::random_string(10));

        modal.id(&id).body(body).xlarge()
    }

    /// Set the modal's id.
    pub fn id(mut self, id: &str) -> Modal {
        self.id = id.into();
        self
    }

    /// Set the modal's body.
    pub fn body(mut self, body: Component) -> Modal {
        self.body = body;
        self
    }

    /// Make the modal x-large.
    pub fn xlarge(mut self) -> Modal {
        self.size_class = "modal-xl".into();
        self
    }

    /// Set the modal's header.
    pub fn header(mut self, header: Component) -> Modal {
        self.header = Some(header);
        self
    }

    // Quick implimitation to toggle the modal
    pub fn toggle(id: &str) -> String {
        format!(r#"data-bs-toggle="modal" data-bs-target="{}{}""#, "#", id)
    }

    pub fn dismiss() -> String {
        r#"data-bs-dismiss="modal""#.into()
    }

    pub fn no_default_style(mut self) -> Modal {
        self.default_style = false;
        self
    }

    pub fn set_static_backdrop(mut self, set_static: bool) -> Modal {
        if set_static {
            self.static_backdrop = r#"data-bs-backdrop="static""#.into();
        } else {
            self.static_backdrop = String::new();
        }
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_modal_with_string() {
        let modal = Modal::new("some random string".into());
        let rendering = modal.render_once().unwrap();

        assert!(rendering.contains("some random string"));
    }
}
