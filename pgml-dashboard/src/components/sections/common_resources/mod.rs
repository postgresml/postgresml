use pgml_components::component;
use sailfish::TemplateOnce;

pub enum Cards {
    Contribute,
    Docs,
    Blog,
    Community,
}

struct Card {
    title: String,
    href: String,
    info: String,
    image: Option<String>,
}

#[derive(TemplateOnce, Default)]
#[template(path = "sections/common_resources/template.html")]
pub struct CommonResources {
    show: Vec<Card>,
}

impl CommonResources {
    pub fn new() -> CommonResources {
        CommonResources {
            show: Vec::from([
                CommonResources::docs_card(),
                CommonResources::blog_card(),
                CommonResources::community_card(),
            ]),
        }
    }

    pub fn show(mut self, cards: Vec<Cards>) -> CommonResources {
        if cards.len() == 3 {
            self.show = Vec::new();
            for item in cards {
                match item {
                    Cards::Blog => self.show.push(CommonResources::blog_card()),
                    Cards::Docs => self.show.push(CommonResources::docs_card()),
                    Cards::Contribute => self.show.push(CommonResources::contribute_card()),
                    _ => self.show.push(CommonResources::community_card()),
                }
            }
        } else {
            self.show = Vec::from([
                CommonResources::docs_card(),
                CommonResources::blog_card(),
                CommonResources::community_card(),
            ])
        }
        self
    }

    fn blog_card() -> Card {
        Card {
            title: "Blog".to_string(),
            href: "/blog".to_string(),
            info: "Get the latest product updates and guides to help build your leading AI application.".to_string(),
            image: None,
        }
    }

    fn docs_card() -> Card {
        Card {
            title: "Docs".to_string(),
            href: "/docs".to_string(),
            info: "Get started with our dev-friendly documentation.".to_string(),
            image: None,
        }
    }

    fn contribute_card() -> Card {
        Card {
            title: "Contribute".to_string(),
            href: "https://github.com/postgresml/postgresml".to_string(),
            info:
                "We’re open-source in every way. Contribute on GitHub or contact us to write a guest post on our blog."
                    .to_string(),
            image: Some("/dashboard/static/images/brands/github-sign-on-light.svg".to_string()),
        }
    }

    fn community_card() -> Card {
        Card {
            title: "Community".to_string(),
            href: "https://discord.gg/DmyJP3qJ7U".to_string(),
            info: "We’re active on our Discord. Connect with the team and fellow PostgresML builders.".to_string(),
            image: Some("/dashboard/static/images/icons/discord-white.svg".to_string()),
        }
    }
}

component!(CommonResources);
