use crate::components::{StaticNav, StaticNavLink};
use crate::models::Cluster;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/navbar/web_app/template.html")]
pub struct WebApp {
    pub links: Vec<StaticNavLink>,
    pub deployment_controls: StaticNav,
    pub cluster: Cluster,
}

impl WebApp {
    pub fn new(links: Vec<StaticNavLink>, deployment_controls: StaticNav) -> WebApp {
        WebApp {
            links,
            deployment_controls,
            cluster: Cluster::default(),
        }
    }

    pub fn cluster(mut self, cluster: Cluster) -> Self {
        self.cluster = cluster;
        self
    }
}

component!(WebApp);
