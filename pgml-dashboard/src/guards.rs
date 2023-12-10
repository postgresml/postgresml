use crate::components::sections::footers::marketing_footer::MarketingFooter;
use crate::templates::components::{StaticNav, StaticNavLink};
use once_cell::sync::OnceCell;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use sailfish::TemplateOnce;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};

static POOL: OnceCell<PgPool> = OnceCell::new();

use crate::{models, utils::config, Context, Notification};

#[derive(Debug, Clone, Default)]
pub struct Cluster {
    pub pool: Option<PgPool>,
    pub context: Context,
    pub notifications: Option<Vec<Notification>>,
}

impl Cluster {
    pub fn default(uri: Option<String>) -> Self {
        // Needed for query cancellation
        let max_connections = 2;

        let min_connections = 1;

        Cluster {
            pool: Some(
                POOL.get_or_init(|| {
                    PgPoolOptions::new()
                        .max_connections(max_connections)
                        .idle_timeout(None)
                        .min_connections(min_connections)
                        .after_connect(|conn, _meta| {
                            Box::pin(async move {
                                conn.execute("SET application_name = 'pgml_dashboard';")
                                    .await?;
                                Ok(())
                            })
                        })
                        .connect_lazy(config::database_url())
                        .expect("Default database URL is malformed")
                })
                .clone(),
            ),
            context: Context {
                user: models::User::default(),
                cluster: models::Cluster::default(),
                dropdown_nav: StaticNav {
                    links: vec![
                        StaticNavLink::new("Local".to_string(), "/dashboard".to_string())
                            .active(true),
                    ],
                },
                account_management_nav: StaticNav {
                    links: vec![
                        StaticNavLink::new("Notebooks".to_string(), "/dashboard".to_string()),
                        StaticNavLink::new(
                            "Projects".to_string(),
                            "/dashboard?tab=Projects".to_string(),
                        ),
                        StaticNavLink::new(
                            "Models".to_string(),
                            "/dashboard?tab=Models".to_string(),
                        ),
                        StaticNavLink::new(
                            "Snapshots".to_string(),
                            "/dashboard?tab=Snapshots".to_string(),
                        ),
                        StaticNavLink::new(
                            "Upload data".to_string(),
                            "/dashboard?tab=Upload_Data".to_string(),
                        ),
                        StaticNavLink::new(
                            "PostgresML.org".to_string(),
                            "https://postgresml.org".to_string(),
                        ),
                    ],
                },
                upper_left_nav: StaticNav {
                    links: vec![
                        StaticNavLink::new(
                            "Notebooks".to_string(),
                            "/dashboard?tab=Notebooks".to_string(),
                        )
                        .icon("add_notes")
                        .active(
                            uri.is_some()
                                && (uri.clone().unwrap().starts_with("/dashboard?tab=Notebook")
                                    || uri.clone().unwrap() == "/dashboard"),
                        ),
                        StaticNavLink::new(
                            "Projects".to_string(),
                            "/dashboard?tab=Projects".to_string(),
                        )
                        .icon("library_add")
                        .active(
                            uri.is_some()
                                && uri.clone().unwrap().starts_with("/dashboard?tab=Project"),
                        ),
                        StaticNavLink::new(
                            "Models".to_string(),
                            "/dashboard?tab=Models".to_string(),
                        )
                        .icon("space_dashboard")
                        .active(
                            uri.is_some()
                                && uri.clone().unwrap().starts_with("/dashboard?tab=Model"),
                        ),
                        StaticNavLink::new(
                            "Snapshots".to_string(),
                            "/dashboard?tab=Snapshots".to_string(),
                        )
                        .icon("filter_center_focus")
                        .active(
                            uri.is_some()
                                && uri.clone().unwrap().starts_with("/dashboard?tab=Snapshot"),
                        ),
                        StaticNavLink::new(
                            "Upload data".to_string(),
                            "/dashboard?tab=Upload_Data".to_string(),
                        )
                        .icon("upload")
                        .active(
                            uri.is_some()
                                && uri
                                    .clone()
                                    .unwrap()
                                    .starts_with("/dashboard?tab=Upload_Data"),
                        ),
                    ],
                },
                lower_left_nav: StaticNav::default(),
                marketing_footer: MarketingFooter::new().render_once().unwrap(),
                head_items: None,
            },
            notifications: None,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r Cluster {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let uri = request.uri().to_string();
        request::Outcome::Success(request.local_cache(|| Cluster::default(Some(uri))))
    }
}

#[derive(Debug)]
pub struct ConnectedCluster<'a> {
    pub inner: &'a Cluster,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ConnectedCluster<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let cluster = match request.guard::<&Cluster>().await {
            request::Outcome::Success(cluster) => cluster,
            _ => return request::Outcome::Forward(Status::NotFound),
        };

        if cluster.pool.as_ref().is_some() {
            request::Outcome::Success(ConnectedCluster { inner: cluster })
        } else {
            request::Outcome::Forward(Status::NotFound)
        }
    }
}

impl<'a> Cluster {
    pub fn pool(&'a self) -> &'a PgPool {
        self.pool.as_ref().unwrap()
    }
}

impl<'a> ConnectedCluster<'_> {
    pub fn pool(&'a self) -> &'a PgPool {
        self.inner.pool()
    }
}
