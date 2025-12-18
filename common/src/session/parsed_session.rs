use crate::AppError;
use axum::http::{HeaderMap, header};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ParsedSession {
    pub ssid: String, // SSID={}
    pub unsigned_ssid: Uuid,
    pub user_id: Uuid,
}

pub enum ParsedSessionError {
    NoCookieHeader,
    ParseError,
    VerificationError,
}

impl ParsedSession {
    /// parses all the cookies sent by a client and creates an `ActiveSession`
    pub fn parse_and_verify(cookies_list: &Vec<String>) -> Result<Self, ParsedSessionError> {
        if cookies_list.is_empty() {
            return Err(ParsedSessionError::NoCookieHeader);
        }
        let mut ssid = None;
        let mut unsigned_ssid = None;
        let mut uuid = None;
        for cookie_header in cookies_list {
            for cookie in cookie_header.split(';') {
                if cookie.trim().starts_with("SSID=") {
                    ssid = Some(cookie.trim()[5..].to_string());
                    unsigned_ssid = Some(
                        Uuid::from_str(
                            &super::verify(&cookie.trim()[5..])
                                .ok_or(ParsedSessionError::VerificationError)?,
                        )
                        .map_err(|_| ParsedSessionError::VerificationError)?,
                    );
                }
                if cookie.trim().starts_with("UUID=") {
                    uuid = Some(
                        uuid::Uuid::from_str(&cookie.trim()[5..])
                            .map_err(|_| ParsedSessionError::VerificationError)?,
                    );
                }
            }
        }
        if let Some(ssid) = ssid
            && let Some(unsigned_ssid) = unsigned_ssid
            && let Some(user_id) = uuid
        {
            Ok(Self { ssid, unsigned_ssid, user_id })
        } else {
            Err(ParsedSessionError::ParseError)
        }
    }

    pub fn parse_and_verify_from_headers(headers: &HeaderMap) -> Result<Self, ParsedSessionError> {
        // collecting all the user sent cookie headers into `cookies_list`
        let cookies_list = headers
            .get_all(header::COOKIE)
            .iter()
            .map(|h| h.to_str().unwrap_or_default().to_string())
            .collect::<Vec<String>>();
        Self::parse_and_verify(&cookies_list)
    }
}

impl From<ParsedSessionError> for AppError {
    fn from(value: ParsedSessionError) -> Self {
        match value {
            ParsedSessionError::NoCookieHeader => AppError::Unauthorized("No cookie header found"),
            ParsedSessionError::ParseError => AppError::InvalidSession(HeaderMap::new()),
            ParsedSessionError::VerificationError => {
                AppError::InvalidSession(super::expire_session())
            }
        }
    }
}
