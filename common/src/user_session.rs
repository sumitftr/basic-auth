use cookie::{Cookie, CookieJar};
use mongodb::bson::datetime::DateTime;

/// this is a wrapper type around `CookieJar`, since it doesn't implements `Serialize` trait
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct UserSession {
    pub cookie: Vec<String>,
    pub created: DateTime,
    pub user_agent: String,
}

impl UserSession {
    pub fn new(user_agent: String) -> Self {
        let mut jar = CookieJar::new();
        // session id cookie
        let ssid = Cookie::build(("SSID", uuid::Uuid::new_v4().to_string()))
            .path("/")
            .max_age(cookie::time::Duration::days(30))
            .same_site(cookie::SameSite::Strict)
            .secure(true)
            .build();
        // creating final cookie jar
        jar.add(ssid);

        Self {
            cookie: jar.delta().map(|c| c.to_string()).collect(),
            created: DateTime::now(),
            user_agent,
        }
    }
}
