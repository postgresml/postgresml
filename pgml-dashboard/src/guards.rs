use std::env::var;

use rocket::http::CookieJar;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use sqlx::PgPool;

use std::collections::HashMap;

use crate::models::User;
use crate::{Clusters, Context, CurrentUser};

pub fn default_database_url() -> String {
    match var("DATABASE_URL") {
        Ok(val) => val,
        Err(_) => "postgres:///pgml".to_string(),
    }
}

#[derive(Debug)]
pub struct Cluster {
    pool: Option<PgPool>,
    pub context: Context,
}

impl<'a> Cluster {
    pub fn pool(&'a self) -> &'a PgPool {
        self.pool.as_ref().unwrap()
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

        let clusters = match request.guard::<&State<Clusters>>().await {
            Outcome::Success(pool) => pool,
            _ => return Outcome::Forward(()),
        };

        let pool = clusters.get(cluster_id);

        let context = Context {
            cluster: clusters.get_context(cluster_id).cluster,
        };

        Outcome::Success(Cluster { pool, context })
    }
}

#[derive(Debug)]
pub struct CurrentUserState {
    pub user: User,
    pub visible_clusters: HashMap<String, String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CurrentUserState {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<CurrentUserState, ()> {
        let user_data = match request.guard::<&State<CurrentUser>>().await {
            Outcome::Success(user) => user,
            _ => return Outcome::Forward(()),
        };

        let current_user_state = CurrentUserState {
            user: user_data.get_user(),
            visible_clusters: user_data.get_visible_clusters(),
        };

        Outcome::Success(current_user_state)
    }
}
