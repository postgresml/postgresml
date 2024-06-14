use chrono;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationCookie {
    pub id: String,
    pub time_viewed: Option<chrono::DateTime<chrono::Utc>>,
    pub time_modal_viewed: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct Notifications {}

impl Notifications {
    pub fn update_viewed(new: &Vec<NotificationCookie>, cookies: &CookieJar<'_>) {
        let serialized = new
            .iter()
            .map(|x| serde_json::to_string(x).unwrap())
            .collect::<Vec<String>>();

        let mut cookie = Cookie::new("session", format!(r#"{{"notifications": [{}]}}"#, serialized.join(",")));
        cookie.set_max_age(::time::Duration::weeks(4));
        cookies.add_private(cookie);
    }

    pub fn get_viewed(cookies: &CookieJar<'_>) -> Vec<NotificationCookie> {
        let viewed: Vec<NotificationCookie> = match cookies.get_private("session") {
            Some(session) => {
                match serde_json::from_str::<serde_json::Value>(session.value())
                    .unwrap_or_else(|_| serde_json::from_str::<serde_json::Value>(r#"{"notifications": []}"#).unwrap())
                    ["notifications"]
                    .as_array()
                {
                    Some(items) => items
                        .into_iter()
                        .map(|x| {
                            serde_json::from_str::<NotificationCookie>(&x.to_string()).unwrap_or_else(|_| {
                                serde_json::from_str::<String>(&x.to_string())
                                    .and_then(|z| {
                                        Ok(NotificationCookie {
                                            id: z,
                                            time_viewed: None,
                                            time_modal_viewed: None,
                                        })
                                    })
                                    .unwrap_or_else(|_| NotificationCookie {
                                        id: "".to_string(),
                                        time_viewed: None,
                                        time_modal_viewed: None,
                                    })
                            })
                        })
                        .collect::<Vec<NotificationCookie>>(),
                    _ => vec![],
                }
            }
            None => vec![],
        };

        viewed
    }
}
