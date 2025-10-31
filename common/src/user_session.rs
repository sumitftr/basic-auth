use crate::AppError;
use axum::http::{HeaderMap, HeaderValue, header};
use base64::Engine;
use hmac::Mac;
use std::time::{Duration, SystemTime};

/// this function creates a session that is passed to the user
/// and stored in both in-memory and primary database
pub fn create_session(user_agent: String) -> (UserSession, ActiveUserSession, HeaderMap) {
    #[allow(clippy::identity_op)]
    let expires = 30 * 86400 + 0; // days * 86400 + secs
    let uid = uuid::Uuid::new_v4().to_string();
    let signed_uid = sign(&uid);

    (
        UserSession {
            unsigned_ssid: uid.clone(),
            expires: SystemTime::now() + Duration::from_secs(expires),
            user_agent,
        },
        ActiveUserSession {
            ssid: format!("{signed_uid}{uid}"),
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

/// finds the current used session from a list of `UserSession`s
pub fn get_session_index(
    sessions: &[UserSession],
    decrypted_ssid: String,
) -> Result<usize, AppError> {
    for (i, session) in sessions.iter().enumerate() {
        if session.unsigned_ssid.contains(&decrypted_ssid) {
            return Ok(i);
        }
    }
    Err(AppError::AuthError("Session not found"))
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UserSession {
    pub unsigned_ssid: String,
    pub expires: SystemTime,
    pub user_agent: String,
}

pub enum UserSessionStatus {
    Valid(u64),
    Expiring(u64),
    Refreshable(u64),
    Invalid,
}

impl UserSession {
    // timestamp in seconds
    pub const MEM_CACHE_DURATION: u64 = 10800; // 3 hours
    pub const MAX_REFRESH_DURATION: u64 = 604800; // 7 days

    /// returns the timestamp difference of the session with current time
    pub fn session_status(&self) -> UserSessionStatus {
        let diff = self
            .expires
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

        if diff > 0 {
            if diff > Self::MEM_CACHE_DURATION as i64 {
                UserSessionStatus::Valid(diff as u64)
            } else {
                UserSessionStatus::Expiring(diff as u64)
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if -diff < Self::MAX_REFRESH_DURATION as i64 {
                UserSessionStatus::Refreshable(diff as u64)
            } else {
                UserSessionStatus::Invalid
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ActiveUserSession {
    pub ssid: String, // SSID=
}

impl ActiveUserSession {
    /// parses all the cookies sent by a client and creates an `ActiveUserSession`
    pub fn parse(cookies_list: &Vec<String>) -> Result<Self, AppError> {
        for cookies in cookies_list {
            if let Some(s) = cookies.split(';').find(|s| s.trim().starts_with("SSID=")) {
                return Ok(Self {
                    ssid: s.trim()[5..].to_string(),
                });
            }
        }
        Err(AppError::InvalidSession(HeaderMap::new()))
    }

    pub fn verify(&self) -> Result<String, AppError> {
        if let Some(s) = verify(&self.ssid) {
            Ok(s)
        } else {
            Err(AppError::InvalidSession(self.expire()))
        }
    }

    pub fn expire(&self) -> HeaderMap {
        HeaderMap::from_iter([(
            header::SET_COOKIE,
            HeaderValue::from_str(&format!(
                "SSID={}; HttpOnly; SameSite=Strict; Secure; Path=/; Max-Age=0",
                self.ssid
            ))
            .unwrap(),
        )])
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
