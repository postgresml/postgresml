use chrono;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{Deserialize, Serialize};
use time::Duration;

/// Session data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notifications {
    /// App-wide notifications.
    notifications: Vec<NotificationCookie>,
}

/// App-wide notifications.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct NotificationCookie {
    /// Unique ID of the notification.
    pub id: String,
    /// TODO: document
    pub time_viewed: Option<chrono::DateTime<chrono::Utc>>,
    /// TODO: document
    pub time_modal_viewed: Option<chrono::DateTime<chrono::Utc>>,
}

/// Previous session state covering only which notifications were viewed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationsCookieOld {
    pub notifications: Vec<String>,
}

impl From<NotificationsCookieOld> for NotificationCookie {
    fn from(old: NotificationsCookieOld) -> Self {
        NotificationCookie {
            id: old.notifications[0].clone(),
            time_viewed: None,
            time_modal_viewed: None,
        }
    }
}

impl Notifications {
    /// Update the viewed notifications in the session.
    pub fn update_viewed(notifications: &[NotificationCookie], cookies: &CookieJar<'_>) {
        let session = Notifications::safe_serialize_session(notifications);

        let mut cookie = Cookie::new("session", session);
        cookie.set_max_age(Duration::weeks(52 * 100)); // Keep the cookie "forever"
        cookies.add_private(cookie);
    }

    /// Get viewed notifications from the session.
    pub fn get_viewed(cookies: &CookieJar<'_>) -> Vec<NotificationCookie> {
        match cookies.get_private("session") {
            Some(session) => Notifications::safe_deserialize_session(session.value()),
            None => vec![],
        }
    }

    pub fn safe_deserialize_session(session: &str) -> Vec<NotificationCookie> {
        match serde_json::from_str::<Notifications>(session) {
            Ok(notifications) => notifications.notifications,
            Err(_) => match serde_json::from_str::<NotificationsCookieOld>(session) {
                Ok(notifications) => vec![NotificationCookie::from(notifications)],
                Err(_) => vec![],
            },
        }
    }

    pub fn safe_serialize_session(notifications: &[NotificationCookie]) -> String {
        let notifications = Notifications {
            notifications: notifications.to_vec(),
        };

        serde_json::to_string(&notifications).unwrap()
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
        let expected = r#"{"notifications":[{"id":"1","time_viewed":null,"time_modal_viewed":null}]}"#;
        assert_eq!(Notifications::safe_serialize_session(&cookies), expected);
    }
}
