use chrono;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct NotificationCookie {
    pub id: String,
    pub time_viewed: Option<chrono::DateTime<chrono::Utc>>,
    pub time_modal_viewed: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct Notifications {}

impl Notifications {
    pub fn update_viewed(all_desired_notifications: &Vec<NotificationCookie>, cookies: &CookieJar<'_>) {
        let session = Notifications::safe_serialize_session(all_desired_notifications);

        let mut cookie = Cookie::new("session", session);
        cookie.set_max_age(::time::Duration::weeks(4));
        cookies.add_private(cookie);
    }

    pub fn get_viewed(cookies: &CookieJar<'_>) -> Vec<NotificationCookie> {
        match cookies.get_private("session") {
            Some(session) => Notifications::safe_deserialize_session(session.value()),
            None => vec![],
        }
    }

    pub fn safe_deserialize_session(session: &str) -> Vec<NotificationCookie> {
        match serde_json::from_str::<serde_json::Value>(session).unwrap_or_else(|_| {
            serde_json::from_str::<serde_json::Value>(&Notifications::safe_serialize_session(&vec![])).unwrap()
        })["notifications"]
            .as_array()
        {
            Some(items) => items
                .into_iter()
                .map(|notification| {
                    serde_json::from_str::<NotificationCookie>(&notification.to_string()).unwrap_or_else(|_| {
                        serde_json::from_str::<String>(&notification.to_string())
                            .and_then(|id| {
                                Ok(NotificationCookie {
                                    id,
                                    time_viewed: None,
                                    time_modal_viewed: None,
                                })
                            })
                            .unwrap_or_else(|_| NotificationCookie::default())
                    })
                })
                .collect::<Vec<NotificationCookie>>(),
            _ => vec![],
        }
    }

    pub fn safe_serialize_session(cookies: &Vec<NotificationCookie>) -> String {
        let serialized = cookies
            .iter()
            .map(|x| serde_json::to_string(x))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<String>>();

        format!(r#"{{"notifications": [{}]}}"#, serialized.join(","))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Test that we can safely deserialize expected session data.
    #[test]
    fn test_safe_deserialize_session() {
        let session = r#"{"notifications": [{"id": "1", "time_viewed": null, "time_modal_viewed": null}, {"id": "1234567891234", "time_viewed": "2021-08-01T00:00:00Z"}]}"#;
        let expected = vec![
            NotificationCookie {
                id: "1".to_string(),
                time_viewed: None,
                time_modal_viewed: None,
            },
            NotificationCookie {
                id: "1234567891234".to_string(),
                time_viewed: Some(
                    chrono::DateTime::parse_from_rfc3339("2021-08-01T00:00:00Z")
                        .unwrap()
                        .into(),
                ),
                time_modal_viewed: None,
            },
        ];
        assert_eq!(Notifications::safe_deserialize_session(session), expected);
    }

    // Test that new notification system is backwards compatible.
    #[test]
    fn test_safe_deserialize_session_old_form() {
        let session = r#"{"notifications": ["123456789"]}"#;
        let expected = vec![NotificationCookie {
            id: "123456789".to_string(),
            time_viewed: None,
            time_modal_viewed: None,
        }];
        assert_eq!(Notifications::safe_deserialize_session(session), expected);
    }

    #[test]
    fn test_safe_deserialize_session_empty() {
        let session = r#"{}"#;
        let expected: Vec<NotificationCookie> = vec![];
        assert_eq!(Notifications::safe_deserialize_session(session), expected);
    }

    #[test]
    fn test_safe_serialize_session() {
        let cookies = vec![NotificationCookie {
            id: "1".to_string(),
            time_viewed: None,
            time_modal_viewed: None,
        }];
        let expected = r#"{"notifications": [{"id":"1","time_viewed":null,"time_modal_viewed":null}]}"#;
        assert_eq!(Notifications::safe_serialize_session(&cookies), expected);
    }
}
