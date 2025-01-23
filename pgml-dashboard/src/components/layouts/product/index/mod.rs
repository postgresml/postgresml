use pgml_components::component;
use sailfish::TemplateOnce;

use pgml_components::Component;

pub use crate::components::{self, cms::index_link::IndexLink, NavLink, StaticNav, StaticNavLink};
use crate::{Notification, NotificationLevel};
use components::notifications::product::ProductBanner;

use crate::components::layouts::Head;
use crate::models::Cluster;
use crate::components::breadcrumbs::Breadcrumbs;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "layouts/product/index/template.html")]
pub struct Index<'a> {
    pub content: Option<String>,
    pub breadcrumbs: Breadcrumbs<'a>,
    pub head: Head,
    pub dropdown_nav: StaticNav,
    pub product_left_nav: crate::components::navigation::left_nav::web_app::Menu,
    pub body_components: Vec<Component>,
    pub cluster: Cluster,
    pub product_banners_high: Vec<ProductBanner>,
    pub product_banner_medium: ProductBanner,
    pub product_banner_marketing: ProductBanner,
}

impl<'a> Index<'a> {
    pub fn new(title: &str, context: &crate::guards::Cluster) -> Self {
        let head = Head::new().title(title).context(&context.context.head_items);
        let cluster = context.context.cluster.clone();

        let all_product_high_level = context
            .notifications
            .clone()
            .unwrap_or_else(|| vec![])
            .into_iter()
            .filter(|n: &Notification| n.level == NotificationLevel::ProductHigh)
            .enumerate()
            .map(|(i, n)| ProductBanner::from_notification(Some(&n)).set_show_modal_on_load(i == 0))
            .collect::<Vec<ProductBanner>>();

        Index {
            head,
            cluster,
            dropdown_nav: context.context.deployment_dropdown.clone(),
            product_left_nav: context.context.product_left_nav.clone(),
            product_banners_high: all_product_high_level,
            product_banner_medium: ProductBanner::from_notification(Notification::next_product_of_level(
                context,
                NotificationLevel::ProductMedium,
            )),
            product_banner_marketing: ProductBanner::from_notification(Notification::next_product_of_level(
                context,
                NotificationLevel::ProductMarketing,
            )),
            body_components: context.context.body_components.clone(),
            ..Default::default()
        }
    }

    pub fn breadcrumbs(&mut self, breadcrumbs: Vec<NavLink<'a>>) -> &mut Self {
        self.breadcrumbs.path = breadcrumbs.clone();
        self
    }

    pub fn disable_upper_nav(&mut self) -> &mut Self {
        let links: Vec<StaticNavLink> = self
            .product_left_nav
            .items
            .links
            .iter()
            .map(|item| item.to_owned().disabled(true))
            .collect();
        self.product_left_nav = crate::components::navigation::left_nav::web_app::Menu {
            back: self.product_left_nav.back.clone(),
            items: StaticNav { links },
        };
        self
    }

    pub fn content(&mut self, content: &str) -> &mut Self {
        self.content = Some(content.to_owned());
        self
    }

    pub fn body_components(&mut self, components: Vec<Component>) -> &mut Self {
        self.body_components.extend(components);
        self
    }

    pub fn breadcrumbs_from_uri(
                &mut self,
                org_dropdown: Vec<StaticNavLink>,
                database_dropdown: Vec<StaticNavLink>,
                uri: &str,
            ) -> &mut Self {
                let uri = if uri.starts_with("/") {
                    uri.chars().skip(1).collect::<String>()
                } else {
                    uri.to_string()
                };
        
                let start_index = match (org_dropdown.is_empty(), database_dropdown.is_empty()) {
                    (true, true) => 0,
                    (false, true) => 1,
                    _ => 2,
                };
        
                let mut uris = uri
                    .split("/")
                    .skip(start_index)
                    .enumerate()
                    .map(|(i, part)| {
                        let path = uri
                            .split("/")
                            .into_iter()
                            .take(1 + i + start_index)
                            .collect::<Vec<&str>>()
                            .join("/");
                        let mut out = "/".to_owned();
                        out.push_str(&path);
        
                        NavLink::new(part, &out)
                    })
                    .collect::<Vec<NavLink>>();
        
                if let Some(last) = uris.clone().into_iter().next_back() {
                    uris.pop();
                    uris.push(last.active());
                }
        
                self.breadcrumbs = Breadcrumbs {
                    organizations: org_dropdown,
                    databases: database_dropdown,
                    path: uris,
                };
        
                self
            }

    pub fn render<T>(&mut self, template: T) -> String
    where
        T: sailfish::TemplateOnce,
    {
        self.content = Some(template.render_once().unwrap());
        (*self).clone().into()
    }
}

impl<'a> From<Index<'a>> for String {
    fn from(layout: Index) -> String {
        layout.render_once().unwrap()
    }
}

component!(Index, 'a);
