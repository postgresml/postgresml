use crate::components::layouts::Head;
use crate::components::notifications::marketing::AlertBanner;
use crate::guards::Cluster;
use crate::models::User;
use crate::Notification;
use pgml_components::component;
use sailfish::TemplateOnce;
use std::fmt;

#[derive(Default, Clone)]
pub enum Theme {
    #[default]
    Marketing,
    Docs,
    Product,
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Theme::Marketing => write!(f, "marketing"),
            Theme::Docs => write!(f, "docs"),
            Theme::Product => write!(f, "product"),
        }
    }
}

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "layouts/marketing/base/template.html")]
pub struct Base {
    pub head: Head,
    pub content: Option<String>,
    pub footer: Option<String>,
    pub alert_banner: AlertBanner,
    pub user: Option<User>,
    pub theme: Theme,
    pub no_transparent_nav: bool,
}

impl Base {
    pub fn new(title: &str, context: Option<&Cluster>) -> Base {
        let title = format!("{} - PostgresML", title);

        let (head, footer, user) = match context.as_ref() {
            Some(context) => (
                Head::new().title(&title).context(&context.context.head_items),
                Some(context.context.marketing_footer.clone()),
                Some(context.context.user.clone()),
            ),
            None => (Head::new().title(&title), None, None),
        };

        Base {
            head,
            footer,
            alert_banner: AlertBanner::from_notification(Notification::next_alert(context)),
            user,
            no_transparent_nav: false,
            ..Default::default()
        }
    }

    pub fn from_head(head: Head, context: Option<&Cluster>) -> Self {
        let mut rsp = Base::new("", context);

        let head = match context.as_ref() {
            Some(context) => head.context(&context.context.head_items),
            None => head,
        };

        rsp.head = head;
        rsp
    }

    pub fn footer(mut self, footer: String) -> Self {
        self.footer = Some(footer);
        self
    }

    pub fn content(mut self, content: &str) -> Self {
        self.content = Some(content.to_owned());
        self
    }

    pub fn user(mut self, user: User) -> Self {
        self.user = Some(user);
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn no_transparent_nav(mut self) -> Self {
        self.no_transparent_nav = true;
        self
    }

    pub fn render<T>(mut self, template: T) -> String
    where
        T: sailfish::TemplateOnce,
    {
        self.content = Some(template.render_once().unwrap());
        self.clone().into()
    }
}

impl From<Base> for String {
    fn from(layout: Base) -> String {
        layout.render_once().unwrap()
    }
}

component!(Base);
