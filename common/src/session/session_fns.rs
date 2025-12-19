use super::{ParsedSession, Session};
use axum::http::{HeaderMap, HeaderValue, header};
use std::time::Duration;
use time::OffsetDateTime;

/// this function creates a session that is passed to the user
/// and stored in both in-memory and primary database
pub fn create_session(
    user_id: uuid::Uuid,
    headers: &HeaderMap,
    socket_addr: std::net::SocketAddr,
) -> (Session, ParsedSession, HeaderMap) {
    let user_agent =
        headers.get(header::USER_AGENT).map(|v| v.to_str().unwrap_or_default().to_owned());

    let now = OffsetDateTime::now_utc();
    let expires_at = now + Duration::from_secs(37 * 86400 + 60); // days * 86400 + secs
    let uid = uuid::Uuid::new_v4();
    let signed_uid = super::sign(&uid.to_string());

    (
        Session {
            unsigned_ssid: uid,
            user_agent,
            ip_address: socket_addr.ip(),
            created_at: now,
            last_used: now,
            expires_at,
        },
        ParsedSession { ssid: format!("{signed_uid}{uid}"), unsigned_ssid: uid, user_id },
        HeaderMap::from_iter([
            (
                header::SET_COOKIE,
                HeaderValue::from_str(&format!(
                    "SSID={signed_uid}{uid}; HttpOnly; SameSite=Strict; Secure; Path=/; Expires={expires_at}",
                ))
                .unwrap(),
            ),
            (
                header::SET_COOKIE,
                HeaderValue::from_str(&format!(
                    "UUID={user_id}; HttpOnly; SameSite=Strict; Secure; Path=/; Expires={expires_at}",
                ))
                .unwrap(),
            ),
        ]),
    )
}

pub fn expire_session() -> HeaderMap {
    HeaderMap::from_iter([
        (
            header::SET_COOKIE,
            HeaderValue::from_str("SSID={}; HttpOnly; SameSite=Strict; Secure; Path=/; Max-Age=0")
                .unwrap(),
        ),
        (
            header::SET_COOKIE,
            HeaderValue::from_str("UUID={}; HttpOnly; SameSite=Strict; Secure; Path=/; Max-Age=0")
                .unwrap(),
        ),
    ])
}
