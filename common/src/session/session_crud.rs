use super::ActiveSession;
use crate::{AppError, session::Session};
use axum::http::{HeaderMap, HeaderValue, header};
use std::time::{Duration, SystemTime};

/// this function creates a session that is passed to the user
/// and stored in both in-memory and primary database
pub fn create_session(user_agent: String) -> (Session, ActiveSession, HeaderMap) {
    #[allow(clippy::identity_op)]
    let expires = 30 * 86400 + 0; // days * 86400 + secs
    let uid = uuid::Uuid::new_v4().to_string();
    let signed_uid = super::sign(&uid);

    (
        Session {
            unsigned_ssid: uid.clone(),
            expires: SystemTime::now() + Duration::from_secs(expires),
            user_agent,
        },
        ActiveSession {
            ssid: format!("{signed_uid}{uid}"),
            decrypted_ssid: uid.clone(),
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

/// finds the current used session from a list of `UserSession`s
pub fn get_session_index(
    sessions: &[Session],
    active_session: &ActiveSession,
) -> Result<usize, AppError> {
    for (i, session) in sessions.iter().enumerate() {
        if session
            .unsigned_ssid
            .contains(&active_session.decrypted_ssid)
        {
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
        .filter(|s| now < s.expires)
        .collect::<Vec<Session>>();

    let _ = std::mem::replace(sessions, filtered_sessions);
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
