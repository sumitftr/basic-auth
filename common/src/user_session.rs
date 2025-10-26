use crate::AppError;
use axum::http::{HeaderMap, HeaderValue, header};
use base64::Engine;
use hmac::Mac;
use std::time::{Duration, SystemTime};

pub fn create_session(user_agent: String) -> (UserSession, ActiveUserSession, HeaderMap) {
    #[allow(clippy::identity_op)]
    let expires = 30 * 86400 + 0; // days * 86400 + secs
    let uid = uuid::Uuid::new_v4().to_string();
    let signed_uid = sign(&uid);

    (
        UserSession {
            unsigned_values: vec![format!("SSID={uid}")],
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

/// this function is used to sign cookie value to ensure integrity and authenticity
///
/// value is the `VALUE` part of the whole cookie (`KEY=VALUE`)
fn sign(value: &str) -> String {
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&crate::SECRET_KEY).unwrap();
    hmac::Mac::update(&mut mac, value.as_bytes());
    base64::prelude::BASE64_STANDARD.encode(mac.finalize().into_bytes())
}

/// this function is used to verify signed cookie value to ensure integrity and authenticity
///
/// value is the `VALUE` part of the whole cookie (`KEY=VALUE`)
fn verify(value: &str) -> Option<String> {
    const BASE64_DIGEST_LEN: usize = 44;

    if !value.is_char_boundary(BASE64_DIGEST_LEN) {
        return None;
    }

    // Split [MAC | original-value] into its two parts.
    let (digest_str, uid) = value.split_at(BASE64_DIGEST_LEN);
    let digest = base64::prelude::BASE64_STANDARD.decode(digest_str).ok()?;

    // Perform the verification
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&crate::SECRET_KEY).unwrap();
    hmac::Mac::update(&mut mac, uid.as_bytes());
    mac.verify_slice(&digest).map(|_| uid.to_string()).ok()
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UserSession {
    pub unsigned_values: Vec<String>,
    pub expires: SystemTime,
    pub user_agent: String,
}

impl UserSession {
    // returns a reference to `UUID` part of the cookie as string slice
    pub fn ssid_value(&self) -> &str {
        &self.unsigned_values[0][5..self.unsigned_values[0].len()]
    }

    // finds the current used session from a list of `UserSession`s
    pub fn current_session_index(&self, sessions: &[UserSession]) -> Result<usize, AppError> {
        let mut ssid_cookie_i = 0;
        // finding the cookie that starts with SSID
        for (i, cookie) in self.unsigned_values.iter().enumerate() {
            if cookie.starts_with("SSID") {
                ssid_cookie_i = i;
            }
        }
        for (i, session) in sessions.iter().enumerate() {
            if session
                .unsigned_values
                .contains(&self.unsigned_values[ssid_cookie_i])
            {
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
    pub fn parse(cookies: String) -> Self {
        Self {
            cookies: cookies
                .split_ascii_whitespace()
                .filter(|s| s.starts_with("SSID="))
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

    pub fn verify(&self) -> Result<Vec<String>, AppError> {
        let (_key, value) = self.cookies[0].split_at(4); // length of the key (here key = `SSID`)
        if let Some(s) = verify(value) {
            Ok(vec![s])
        } else {
            Err(AppError::InvalidSession(HeaderMap::from_iter([(
                header::SET_COOKIE,
                HeaderValue::from_str(&format!(
                    "{}; HttpOnly; SameSite=Strict; Secure; Path=/; Max-Age=0",
                    self.cookies[0]
                ))
                .unwrap(),
            )])))
        }
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

    #[test]
    fn sign_then_verify() {
        dotenv::dotenv().ok();
        let uid = uuid::Uuid::new_v4().to_string();
        dbg!(&uid);
        let signed_uid = sign(&uid);
        dbg!(&signed_uid);
        let verified_uid = verify(&format!("{signed_uid}{uid}")).unwrap();
        dbg!(&verified_uid);
        assert_eq!(uid, verified_uid);
    }
}
