use crate::AppError;
use axum::http::{HeaderMap, HeaderValue, header};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ActiveSession {
    pub ssid: String, // SSID=
    pub decrypted_ssid: String,
}

impl ActiveSession {
    /// parses all the cookies sent by a client and creates an `ActiveSession`
    pub fn parse_and_verify(cookies_list: &Vec<String>) -> Result<Self, AppError> {
        if cookies_list.is_empty() {
            return Err(AppError::Unauthorized("No cookie header found"));
        }
        let mut parsed_active_session = None;
        for cookies in cookies_list {
            if let Some(s) = cookies.split(';').find(|s| s.trim().starts_with("SSID=")) {
                parsed_active_session = Some(Self {
                    ssid: s.trim()[5..].to_string(),
                    decrypted_ssid: "".to_string(),
                });
                break;
            }
        }
        if let Some(mut active_session) = parsed_active_session {
            if let Some(s) = super::verify(&active_session.ssid) {
                active_session.decrypted_ssid = s;
                Ok(active_session)
            } else {
                Err(AppError::InvalidSession(active_session.expire()))
            }
        } else {
            Err(AppError::Unauthorized("Invalid Session"))
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
