use rocket::http::{CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use sqlx::PgPool;

use std::env::var;

use crate::{Clusters, Context};

#[derive(Debug)]
pub struct Cluster {
    pool: PgPool,
    pub context: Context,
}

impl<'a> Cluster {
    pub fn pool(&'a self) -> &'a PgPool {
        &self.pool
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Cluster {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Cluster, ()> {
        // Using `State` as a request guard. Use `inner()` to get an `'r`.
        let cookies = match request.guard::<&CookieJar<'r>>().await {
            Outcome::Success(cookies) => cookies,
            _ => return Outcome::Forward(()),
        };

        let cluster_id = match cookies.get_private("cluster_id") {
            Some(cluster_id) => match cluster_id.value().parse::<i64>() {
                Ok(cluster_id) => cluster_id,
                Err(_) => -1,
            },

            None => -1,
        };

        let shared_state = match request.guard::<&State<Clusters>>().await {
            Outcome::Success(pool) => pool,
            _ => return Outcome::Forward(()),
        };

        let pool = match shared_state.get(cluster_id) {
            Some(pool) => pool,
            None => return Outcome::Failure((Status::BadRequest, ())),
        };

        Outcome::Success(Cluster { pool, context: shared_state.get_context(cluster_id) })
    }
}

pub fn default_database_url() -> String {
    match var("DATABASE_URL") {
        Ok(val) => val,
        Err(_) => "postgres:///dashboard".to_string(),
    }
}
