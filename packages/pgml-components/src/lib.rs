//! A basic UI component. Any other component can accept this
//! as a parameter and render it.

use sailfish::TemplateOnce;

#[derive(Default, Clone, TemplateOnce)]
#[template(path = "components/component.html")]
pub struct Component {
    pub value: String,
}

#[macro_export]
macro_rules! component {
    ($name:tt) => {
        impl From<$name> for pgml_components::Component {
            fn from(thing: $name) -> pgml_components::Component {
                use sailfish::TemplateOnce;

                pgml_components::Component {
                    value: thing.render_once().unwrap(),
                }
            }
        }
    };

    ($name:tt, $lifetime:lifetime) => {
        impl<$lifetime> From<$name<$lifetime>> for pgml_components::Component {
            fn from(thing: $name<$lifetime>) -> pgml_components::Component {
                use sailfish::TemplateOnce;

                pgml_components::Component {
                    value: thing.render_once().unwrap(),
                }
            }
        }
    };
}

// pub use component;

// Render any string.
impl From<&str> for Component {
    fn from(value: &str) -> Component {
        Component {
            value: value.to_owned(),
        }
    }
}

// Render any string.
impl From<String> for Component {
    fn from(value: String) -> Component {
        Component { value }
    }
}
