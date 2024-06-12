use chrono;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationCookie {
    pub id: String,
    pub time_viewed: Option<chrono::DateTime<chrono::Utc>>,
    pub time_modal_viewed: Option<chrono::DateTime<chrono::Utc>>,
}

impl std::fmt::Display for NotificationCookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rsp = format!(r#"{{"id": "{}""#, self.id.clone());
        if self.time_viewed.is_some() {
            rsp.push_str(&format!(r#", "time_viewed": "{}""#, self.time_viewed.clone().unwrap()));
        }
        if self.time_modal_viewed.is_some() {
            rsp.push_str(&format!(
                r#", "time_modal_viewed": "{}""#,
                self.time_modal_viewed.clone().unwrap()
            ));
        }
        rsp.push_str("}}");
        println!("rsp: {}", rsp);
        return write!(f, "{}", rsp);
    }
}

pub struct Notifications {}

impl Notifications {
    pub fn update_viewed(new: &Vec<NotificationCookie>, cookies: &CookieJar<'_>) {
        let serialized = new.iter().map(|x| x.to_string()).collect::<Vec<String>>();

        let mut cookie = Cookie::new("session", format!(r#"{{"notifications": [{}]}}"#, serialized.join(",")));
        cookie.set_max_age(::time::Duration::weeks(4));
        cookies.add_private(cookie);
    }

    pub fn get_viewed(cookies: &CookieJar<'_>) -> Vec<NotificationCookie> {
        let viewed: Vec<NotificationCookie> = match cookies.get_private("session") {
            Some(session) => {
                match serde_json::from_str::<serde_json::Value>(session.value()).unwrap()["notifications"].as_array() {
                    Some(items) => items
                        .into_iter()
                        .map(|x| serde_json::from_str::<NotificationCookie>(&x.to_string()).unwrap())
                        .collect::<Vec<NotificationCookie>>(),
                    _ => vec![],
                }
            }
            None => vec![],
        };

        viewed
    }
}
