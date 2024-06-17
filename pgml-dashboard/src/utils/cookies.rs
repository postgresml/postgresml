use rocket::http::{Cookie, CookieJar};

pub struct Notifications {}

impl Notifications {
    pub fn update_viewed(new: &Vec<String>, cookies: &CookieJar<'_>) {
        let mut cookie = Cookie::new("session", format!(r#"{{"notifications": {:?}}}"#, new));
        cookie.set_max_age(::time::Duration::weeks(4));
        cookies.add_private(cookie);
    }

    pub fn get_viewed(cookies: &CookieJar<'_>) -> Vec<String> {
        let viewed = match cookies.get_private("session") {
            Some(session) => {
                match serde_json::from_str::<serde_json::Value>(session.value()).unwrap()["notifications"].as_array() {
                    Some(items) => items
                        .into_iter()
                        .map(|x| x.as_str().unwrap().to_string())
                        .collect::<Vec<String>>(),
                    _ => vec![],
                }
            }
            None => vec![],
        };

        viewed
    }
}
