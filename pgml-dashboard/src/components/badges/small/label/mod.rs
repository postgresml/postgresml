use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "badges/small/label/template.html")]
pub struct Label {
    value: String,
    image_url: String,
}

impl Label {
    pub fn check_circle(value: &str) -> Label {
        Label {
            value: value.into(),
            image_url: "/dashboard/static/images/icons/check_circle.svg".to_string(),
        }
    }

    pub fn cancel(value: &str) -> Label {
        Label {
            value: value.into(),
            image_url: "/dashboard/static/images/icons/cancel.svg".to_string(),
        }
    }

    pub fn outbound(value: &str) -> Label {
        Label {
            value: value.into(),
            image_url: "/dashboard/static/images/icons/outbound.svg".to_string(),
        }
    }

    pub fn download_for_offline(value: &str) -> Label {
        Label {
            value: value.into(),
            image_url: "/dashboard/static/images/icons/download_for_offline.svg".to_string(),
        }
    }

    pub fn forward_circle(value: &str) -> Label {
        Label {
            value: value.into(),
            image_url: "/dashboard/static/images/icons/forward_circle.svg".to_string(),
        }
    }
}

component!(Label);
