mod cookie;
mod parsed_session;
mod session_fns;
mod session_struct;

pub use cookie::BASE64_DIGEST_LEN;
use cookie::{sign, verify};
pub use parsed_session::{ParsedSession, ParsedSessionError};
pub use session_fns::{create_session, expire_session};
pub use session_struct::{Session, SessionStatus};

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    use axum::http::{HeaderMap, HeaderValue};
    use reqwest::header;
    use std::net::{Ipv4Addr, SocketAddr};

    use super::*;

    #[test]
    fn db_session_test() {
        dotenv::dotenv().ok();
        let headers = HeaderMap::from_iter([(
            header::USER_AGENT,
            HeaderValue::from_str("Mozilla Firefox").unwrap(),
        )]);
        let uid = uuid::Uuid::new_v4();
        let sock_addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 39849);
        let (new_session, parsed_session, set_cookie_headermap) =
            create_session(uid, &headers, sock_addr);
        dbg!(&new_session);
        dbg!(&parsed_session);
        dbg!(&set_cookie_headermap);
        assert!(parsed_session.ssid.ends_with(new_session.unsigned_ssid.to_string().as_str()));
        assert!(
            parsed_session.ssid.starts_with(&sign(new_session.unsigned_ssid.to_string().as_str()))
        );
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

    // #[test]
    // fn syncing_session_test() {
    //     dotenv::dotenv().ok();
    //     let headers = HeaderMap::from_iter([(
    //         header::USER_AGENT,
    //         HeaderValue::from_str("Mozilla Firefox").unwrap(),
    //     )]);
    //     let (new_session1, _, _) = create_session(&headers);
    //     let new_session2 = Session {
    //         unsigned_ssid: create_session(&headers).0.unsigned_ssid,
    //         expires: SystemTime::now() - Duration::from_secs(348738749374),
    //         user_agent: "some agent".to_string(),
    //     };
    //     let (new_session3, _, _) = create_session(&headers);
    //     let new_session4 = Session {
    //         unsigned_ssid: "ertnasotenoariesntoaiesnoa".to_string(),
    //         expires: SystemTime::now() - Duration::from_secs(23829389283),
    //         user_agent: "some agent".to_string(),
    //     };
    //     let mut sessions = vec![new_session1, new_session2, new_session3, new_session4];
    //     clear_expired_sessions(&mut sessions);
    //     assert_eq!(sessions.len(), 2);
    // }
}
