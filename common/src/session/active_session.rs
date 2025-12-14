use crate::AppError;
use axum::http::{HeaderMap, header};
use std::str::FromStr;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ActiveSession {
    pub ssid: String, // SSID={}
    pub unsigned_ssid: String,
    pub user_id: uuid::Uuid,
}

pub enum ActiveSessionError {
    NoCookieHeader,
    ParseError,
    VerificationError,
}

impl ActiveSession {
    /// parses all the cookies sent by a client and creates an `ActiveSession`
    pub fn parse_and_verify(cookies_list: &Vec<String>) -> Result<Self, ActiveSessionError> {
        if cookies_list.is_empty() {
            return Err(ActiveSessionError::NoCookieHeader);
        }
        let mut ssid = None;
        let mut unsigned_ssid = None;
        let mut uuid = None;
        for cookies in cookies_list {
            if let Some(s) = cookies.split(';').find(|s| s.trim().starts_with("SSID=")) {
                ssid = Some(s.trim()[5..].to_string());
                unsigned_ssid = Some(
                    super::verify(&s.trim()[5..]).ok_or(ActiveSessionError::VerificationError)?,
                );
            }
            if let Some(s) = cookies.split(';').find(|s| s.trim().starts_with("UUID=")) {
                uuid = Some(
                    uuid::Uuid::from_str(s).map_err(|_| ActiveSessionError::VerificationError)?,
                );
            }
        }
        if let Some(ssid) = ssid
            && let Some(unsigned_ssid) = unsigned_ssid
            && let Some(user_id) = uuid
        {
            Ok(Self { ssid, unsigned_ssid, user_id })
        } else {
            Err(ActiveSessionError::ParseError)
        }
    }

    pub fn parse_and_verify_from_headers(headers: &HeaderMap) -> Result<Self, ActiveSessionError> {
        // collecting all the user sent cookie headers into `cookies_list`
        let cookies_list = headers
            .get_all(header::COOKIE)
            .iter()
            .map(|h| h.to_str().unwrap_or_default().to_string())
            .collect::<Vec<String>>();
        Self::parse_and_verify(&cookies_list)
    }
}

impl From<ActiveSessionError> for AppError {
    fn from(value: ActiveSessionError) -> Self {
        match value {
            ActiveSessionError::NoCookieHeader => AppError::Unauthorized("No cookie header found"),
            ActiveSessionError::ParseError => AppError::InvalidSession(HeaderMap::new()),
            ActiveSessionError::VerificationError => {
                AppError::InvalidSession(super::expire_session())
            }
        }
    }
}
