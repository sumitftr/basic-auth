use crate::AppError;
use axum::http::{HeaderMap, HeaderValue, header};
use base64::Engine;
use std::time::{Duration, SystemTime};

pub fn create_session(user_agent: String) -> (UserSession, ActiveUserSession, HeaderMap) {
    #[allow(clippy::identity_op)]
    let expires = 30 * 86400 + 0; // days * 86400 + secs
    let uid = uuid::Uuid::new_v4().to_string();
    let signed_uid = sign(&uid);

    (
        UserSession {
            uuids: vec![format!("SSID={uid}")],
            expires: SystemTime::now() + Duration::from_secs(expires),
            user_agent,
        },
        ActiveUserSession {
            cookies: vec![format!("SSID={signed_uid}{uid}")],
        },
        HeaderMap::from_iter([(
            header::SET_COOKIE,
            HeaderValue::from_str(&format!(
                "SSID={signed_uid}{uid}; HttpOnly; SameSite=Strict; Secure; Path=/; Max-Age={expires}",
            ))
            .unwrap(),
        )]),
    )
}

fn sign(value: &str) -> String {
    use hmac::Mac;
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&crate::SECRET_KEY).unwrap();
    hmac::Mac::update(&mut mac, value.as_bytes());
    base64::prelude::BASE64_STANDARD.encode(mac.finalize().into_bytes())
}

fn verify(value: &str) -> bool {
    todo!()
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UserSession {
    pub uuids: Vec<String>,
    pub expires: SystemTime,
    pub user_agent: String,
}

impl UserSession {
    // returns a reference to `UUID` part of the cookie as string slice
    pub fn ssid_value(&self) -> &str {
        &self.uuids[0][5..self.uuids[0].len()]
    }

    // finds the current used session from a list of `UserSession`s
    pub fn current_session_index(&self, sessions: &[UserSession]) -> Result<usize, AppError> {
        let mut ssid_cookie_i = 0;
        // finding the cookie that starts with SSID
        for (i, cookie) in self.uuids.iter().enumerate() {
            if cookie.starts_with("SSID") {
                ssid_cookie_i = i;
            }
        }
        for (i, session) in sessions.iter().enumerate() {
            if session.uuids.contains(&self.uuids[ssid_cookie_i]) {
                return Ok(i);
            }
        }
        Err(AppError::BadReq("Session not found"))
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct ActiveUserSession {
    pub cookies: Vec<String>,
}

impl ActiveUserSession {
    // parses all the cookies sent by a client and creates an `ActiveUserSession`
    pub fn parse_cookies(cookies: String) -> Self {
        Self {
            cookies: cookies
                .split_ascii_whitespace()
                .map(|s| {
                    if let Some(';') = s.chars().last() {
                        s[..s.len() - 1].to_string()
                    } else {
                        s.to_string()
                    }
                })
                .collect::<Vec<String>>(),
        }
    }

    pub fn verify_session() -> Result<(), AppError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    use super::*;

    #[test]
    fn user_session_test() {
        dotenv::dotenv().ok();
        let (user_session, active_user_session, jar) =
            create_session("Mozilla Firefox".to_string());
        dbg!(&user_session);
        dbg!(&active_user_session);
        dbg!(&jar);
        dbg!(&mongodb::bson::DateTime::from_system_time(
            user_session.expires
        ));
        // for i in 0..jar.len() {
        //     assert_eq!(user_session.uuids[i], jar[i][0..jar[i].find(';').unwrap()])
        // }
    }
}
