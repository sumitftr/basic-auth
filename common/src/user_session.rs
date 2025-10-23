use cookie::{Cookie, CookieJar};
use mongodb::bson::datetime::DateTime;

use crate::AppError;

/// this is a wrapper type around `CookieJar`, since it doesn't implements `Serialize` trait
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct UserSession {
    pub cookies: Vec<String>,
    pub created: DateTime,
    pub user_agent: String,
}

impl UserSession {
    pub fn new_with_full(user_agent: String) -> (Self, Vec<String>) {
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

        let full_cookie_jar = jar.delta().map(|c| c.to_string()).collect::<Vec<String>>();
        (
            Self {
                cookies: full_cookie_jar
                    .iter()
                    .map(|c| c[0..c.find(';').unwrap()].to_string())
                    .collect(),
                created: DateTime::now(),
                user_agent,
            },
            full_cookie_jar,
        )
    }

    // returns a reference to `UUID` part of the cookie as string slice
    pub fn ssid(&self) -> &str {
        &self.cookies[0][5..self.cookies[0].len()]
    }

    // finds the current used session from a list of `UserSession`s
    pub fn current_session_index(&self, sessions: &[UserSession]) -> Result<usize, AppError> {
        let mut ssid_cookie_i = 0;
        // finding the cookie that starts with SSID
        for (i, cookie) in self.cookies.iter().enumerate() {
            if cookie.starts_with("SSID") {
                ssid_cookie_i = i;
            }
        }
        for (i, session) in sessions.iter().enumerate() {
            if session.cookies.contains(&self.cookies[ssid_cookie_i]) {
                return Ok(i);
            }
        }
        Err(AppError::BadReq("Session not found"))
    }

    // verifies all the cookies sent by a client in a single request
    pub fn verify_cookies(&self, cookies: String) -> Result<(), AppError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    use super::*;

    #[test]
    fn user_session_test() {
        let (session, jar) = UserSession::new_with_full("Mozilla Firefox".to_string());
        for i in 0..jar.len() {
            dbg!(&jar[i]);
            assert_eq!(session.cookies[i], jar[i][0..jar[i].find(';').unwrap()])
        }
    }
}
