mod cookie;
mod parsed_session;
mod session_crud;
mod session_struct;

pub use cookie::BASE64_DIGEST_LEN;
use cookie::{sign, verify};
pub use parsed_session::{ParsedSession, ParsedSessionError};
pub use session_crud::{
    clear_expired_sessions, create_session, delete_current_session, delete_selected_sessions,
    expire_session, get_session_index,
};
pub use session_struct::{Session, SessionStatus};

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    use axum::http::{HeaderMap, HeaderValue};
    use reqwest::header;

    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn db_session_test() {
        dotenv::dotenv().ok();
        let headers = HeaderMap::from_iter([(
            header::USER_AGENT,
            HeaderValue::from_str("Mozilla Firefox").unwrap(),
        )]);
        let (db_session, parsed_session, set_cookie_headermap) = create_session(&headers);
        dbg!(&db_session);
        dbg!(&parsed_session);
        dbg!(&set_cookie_headermap);
        assert!(parsed_session.ssid.ends_with(db_session.unsigned_ssid.as_str()));
        assert!(parsed_session.ssid.starts_with(&sign(&db_session.unsigned_ssid)));
    }

    #[test]
    fn sign_then_verify() {
        dotenv::dotenv().ok();
        let uid = uuid::Uuid::new_v4().to_string();
        let signed_uid = sign(&uid);
        let decrypted_uid = verify(&format!("{signed_uid}{uid}")).unwrap();
        dbg!(&uid);
        dbg!(&signed_uid);
        dbg!(&decrypted_uid);
        assert_eq!(uid, decrypted_uid);
    }

    #[test]
    fn syncing_session_test() {
        dotenv::dotenv().ok();
        let headers = HeaderMap::from_iter([(
            header::USER_AGENT,
            HeaderValue::from_str("Mozilla Firefox").unwrap(),
        )]);
        let (db_session1, _, _) = create_session(&headers);
        let db_session2 = Session {
            unsigned_ssid: create_session(&headers).0.unsigned_ssid,
            expires: SystemTime::now() - Duration::from_secs(348738749374),
            user_agent: "some agent".to_string(),
        };
        let (db_session3, _, _) = create_session(&headers);
        let db_session4 = Session {
            unsigned_ssid: "ertnasotenoariesntoaiesnoa".to_string(),
            expires: SystemTime::now() - Duration::from_secs(23829389283),
            user_agent: "some agent".to_string(),
        };
        let mut sessions = vec![db_session1, db_session2, db_session3, db_session4];
        clear_expired_sessions(&mut sessions);
        assert_eq!(sessions.len(), 2);
    }
}
