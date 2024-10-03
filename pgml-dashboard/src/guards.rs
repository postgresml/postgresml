use crate::components::sections::footers::marketing_footer::MarketingFooter;
use crate::templates::components::{StaticNav, StaticNavLink};
use crate::utils::urls;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use axum::{async_trait, Extension};
use once_cell::sync::OnceCell;
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
    pub fn default() -> Self {
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
                                conn.execute("SET application_name = 'pgml_dashboard';").await?;
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
                    links: vec![StaticNavLink::new("Local".to_string(), "/dashboard".to_string()).active(true)],
                },
                product_left_nav: StaticNav {
                    links: vec![
                        StaticNavLink::new("Notebooks".to_string(), urls::deployment_notebooks())
                            .icon("format_list_bulleted_add"),
                        StaticNavLink::new("Projects".to_string(), urls::deployment_projects()).icon("library_add"),
                        StaticNavLink::new("Models".to_string(), urls::deployment_models()).icon("grid_view"),
                        StaticNavLink::new("Snapshots".to_string(), urls::deployment_snapshots())
                            .icon("filter_center_focus"),
                        StaticNavLink::new("Upload data".to_string(), urls::deployment_uploader()).icon("upload"),
                    ],
                },
                marketing_footer: MarketingFooter::new().render_once().unwrap(),
                head_items: None,
            },
            notifications: None,
        }
    }
}

// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for &'r Cluster {
//     type Error = ();

//     async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
//         request::Outcome::Success(request.local_cache(|| Cluster::default()))
//     }
// }

#[derive(Debug)]
pub struct ConnectedCluster {
    pub inner: Cluster,
}

// 404 rather than 500 if the cluster is not connected.
// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for ConnectedCluster<'r> {
//     type Error = ();

//     async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
//         let cluster = match request.guard::<&Cluster>().await {
//             request::Outcome::Success(cluster) => cluster,
//             _ => return request::Outcome::Forward(Status::NotFound),
//         };

//         if cluster.pool.as_ref().is_some() {
//             request::Outcome::Success(ConnectedCluster { inner: cluster })
//         } else {
//             request::Outcome::Forward(Status::NotFound)
//         }
//     }
// }

#[async_trait]
impl<S> FromRequestParts<S> for ConnectedCluster
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;

        match parts.extract::<Extension<Cluster>>().await {
            Ok(Extension(cluster)) if cluster.pool.as_ref().is_some() => Ok(ConnectedCluster { inner: cluster }),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

impl Cluster {
    pub fn pool(&self) -> &PgPool {
        self.pool.as_ref().unwrap()
    }
}

impl ConnectedCluster {
    pub fn pool(&self) -> &PgPool {
        self.inner.pool()
    }
}
