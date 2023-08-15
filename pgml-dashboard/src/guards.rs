use std::env::var;

use crate::templates::components::{StaticNav, StaticNavLink};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};

use crate::models;
use crate::{ClustersSettings, Context};

pub fn default_database_url() -> String {
    match var("DATABASE_URL") {
        Ok(val) => val,
        Err(_) => "postgres:///pgml".to_string(),
    }
}

#[derive(Debug)]
pub struct Cluster {
    pub pool: Option<PgPool>,
    pub context: Context,
}

impl Default for Cluster {
    fn default() -> Self {
        let max_connections = 1;
        let min_connections = 1;
        let idle_timeout = 0;

        let settings = ClustersSettings {
            max_connections,
            idle_timeout,
            min_connections,
        };

        Cluster {
            pool: Some(
                PgPoolOptions::new()
                    .max_connections(settings.max_connections)
                    .idle_timeout(std::time::Duration::from_millis(settings.idle_timeout))
                    .min_connections(settings.min_connections)
                    .after_connect(|conn, _meta| {
                        Box::pin(async move {
                            conn.execute("SET application_name = 'pgml_dashboard';")
                                .await?;
                            Ok(())
                        })
                    })
                    .connect_lazy(&default_database_url())
                    .expect("Default database URL is alformed"),
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
                account_management_nav: StaticNav::default(),
                upper_left_nav: StaticNav {
                    links: vec![StaticNavLink::new(
                        "Dashboard".to_string(),
                        "/dashboard".to_string(),
                    )
                    .icon("thumbnail_bar")],
                },
                lower_left_nav: StaticNav::default(),
            },
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r Cluster {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        request::Outcome::Success(request.local_cache(|| Cluster::default()))
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
