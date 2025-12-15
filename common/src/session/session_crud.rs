use super::{ActiveSession, Session};
use crate::AppError;
use axum::http::{HeaderMap, HeaderValue, header};
use std::time::{Duration, SystemTime};
use time::OffsetDateTime;

/// this function creates a session that is passed to the user
/// and stored in both in-memory and primary database
pub fn create_session(
    user_id: uuid::Uuid,
    headers: &HeaderMap,
    socket_addr: std::net::SocketAddr,
) -> (Session, ActiveSession, HeaderMap) {
    let user_agent = headers
        .get(header::USER_AGENT)
        .map(|v| v.to_str().unwrap_or_default().to_owned())
        .unwrap_or_default();

    let now = OffsetDateTime::now_utc();
    let expires_at = now + Duration::from_secs(37 * 86400 + 60); // days * 86400 + secs
    let uid = uuid::Uuid::new_v4().to_string();
    let signed_uid = super::sign(&uid);

    (
        Session {
            unsigned_ssid: uid.clone(),
            user_id,
            user_agent,
            ip_address: socket_addr.ip(),
            created_at: now,
            last_used: now,
            expires_at,
        },
        ActiveSession {
            ssid: format!("{signed_uid}{uid}"),
            unsigned_ssid: uid.clone(),
            user_id,
        },
        HeaderMap::from_iter([(
            header::SET_COOKIE,
            HeaderValue::from_str(&format!(
                "SSID={signed_uid}{uid}; HttpOnly; SameSite=Strict; Secure; Path=/; Expires={expires_at}",
            ))
            .unwrap(),
        ),(
            header::SET_COOKIE,
            HeaderValue::from_str(&format!(
                "UUID={user_id}; HttpOnly; SameSite=Strict; Secure; Path=/; Expires={expires_at}",
            ))
            .unwrap(),
        )
        ]),
    )
}

/// finds the current used session from a list of `UserSession`s
pub fn get_session_index(
    sessions: &[Session],
    active_session: &ActiveSession,
) -> Result<usize, AppError> {
    for (i, session) in sessions.iter().enumerate() {
        if session.unsigned_ssid == active_session.unsigned_ssid {
            return Ok(i);
        }
    }
    Err(AppError::SessionExpired)
}

pub fn clear_expired_sessions(sessions: &mut Vec<Session>) {
    let tmp_sessions = std::mem::take(sessions);

    let now = SystemTime::now();
    let filtered_sessions = tmp_sessions
        .into_iter()
        .filter(|s| now < s.expires_at)
        .collect::<Vec<Session>>();

    let _ = std::mem::replace(sessions, filtered_sessions);
}

pub fn delete_current_session(sessions: &mut Vec<Session>, cur: &ActiveSession) {
    *sessions = sessions.drain(..).filter(|v| v.unsigned_ssid != cur.unsigned_ssid).collect();
}

pub fn delete_selected_sessions(sessions: Vec<Session>, mut selected: Vec<String>) -> Vec<Session> {
    sessions
        .into_iter()
        .filter(|v| {
            #[allow(clippy::needless_range_loop)]
            for i in 0..selected.len() {
                if selected[i] == v.unsigned_ssid {
                    std::mem::take(&mut selected[i]);
                    return false;
                }
            }
            true
        })
        .collect::<Vec<Session>>()
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
