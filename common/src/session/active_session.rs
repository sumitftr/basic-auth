use crate::AppError;
use axum::http::{HeaderMap, HeaderValue, header};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ActiveSession {
    pub ssid: String, // SSID=
    pub decrypted_ssid: String,
}

pub enum ActiveSessionError {
    NoCookieHeader,
    ParseError,
    VerificationError(ActiveSession),
}

impl ActiveSession {
    /// parses all the cookies sent by a client and creates an `ActiveSession`
    pub fn parse_and_verify(cookies_list: &Vec<String>) -> Result<Self, ActiveSessionError> {
        if cookies_list.is_empty() {
            return Err(ActiveSessionError::NoCookieHeader);
        }
        let mut parsed_active_session = None;
        for cookies in cookies_list {
            if let Some(s) = cookies.split(';').find(|s| s.trim().starts_with("SSID=")) {
                parsed_active_session =
                    Some(Self { ssid: s.trim()[5..].to_string(), decrypted_ssid: "".to_string() });
                break;
            }
        }
        if let Some(mut active_session) = parsed_active_session {
            if let Some(s) = super::verify(&active_session.ssid) {
                active_session.decrypted_ssid = s;
                Ok(active_session)
            } else {
                Err(ActiveSessionError::VerificationError(active_session))
            }
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

impl From<ActiveSessionError> for AppError {
    fn from(value: ActiveSessionError) -> Self {
        match value {
            ActiveSessionError::NoCookieHeader => AppError::Unauthorized("No cookie header found"),
            ActiveSessionError::ParseError => AppError::InvalidSession(HeaderMap::new()),
            ActiveSessionError::VerificationError(active_session) => {
                AppError::InvalidSession(active_session.expire())
            }
        }
    }
}
