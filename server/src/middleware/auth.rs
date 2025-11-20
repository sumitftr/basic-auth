use axum::{extract::Request, http::header, middleware::Next, response::Response};
use common::{
    AppError,
    session::{self, ActiveSession, SessionStatus},
};

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    // collecting all the user sent cookie headers into `cookies`
    let cookies = req
        .headers()
        .get_all(header::COOKIE)
        .iter()
        .map(|h| h.to_str().unwrap_or_default().to_string())
        .collect::<Vec<String>>();

    // parsing all user sent cookies to create a valid `ActiveUserSession`
    let active_session = ActiveSession::parse(&cookies)?;

    // checking whether cookies inside `active_session` is tampered or not
    let decrypted_ssid = active_session.verify()?;

    let db = database::Db::new().await;

    // when the use is found inside `Db::active`
    if let Some(user) = db.get_active_user(&active_session) {
        req.extensions_mut().insert(active_session);
        req.extensions_mut().insert(user);
    } else {
        // when the user is not found inside `Db::active`
        match db.get_user_by_decrypted_ssid(&decrypted_ssid).await {
            Ok(mut user) => {
                let i = session::get_session_index(&user.sessions, decrypted_ssid)?;
                match user.sessions[i].session_status() {
                    SessionStatus::Valid(_) => {
                        // adding session and `User` to `Db::active` for faster access
                        let wrapped_user = db.make_user_active(active_session.clone(), user);
                        req.extensions_mut().insert(active_session);
                        req.extensions_mut().insert(wrapped_user);
                    }
                    SessionStatus::Expiring(_) | SessionStatus::Refreshable(_) => {
                        // automatic session refresh code block
                        let user_agent = req
                            .headers()
                            .get(header::USER_AGENT)
                            .map(|v| v.to_str().unwrap_or_default().to_owned())
                            .unwrap_or_default();

                        // creating session
                        let (user_session, new_active_session, set_cookie_headermap) =
                            session::create_session(user_agent);

                        // replacing the old session with new session
                        user.sessions[i] = user_session;

                        // clearing expired sessions
                        session::clear_expired_sessions(&mut user.sessions);

                        // updating session in primary database
                        db.update_sessions(&user.username, &user.sessions).await?;

                        // activating session by adding it to `Db::active`
                        db.make_user_active(new_active_session, user);

                        // in the case of `Expiring` the new ssid will override the old one
                        return Err(AppError::RefreshSession(set_cookie_headermap));
                    }
                    SessionStatus::Invalid => {
                        // clearing expired sessions
                        session::clear_expired_sessions(&mut user.sessions);

                        // updating session in primary database
                        db.update_sessions(&user.username, &user.sessions).await?;

                        return Err(AppError::InvalidSession(active_session.expire()));
                    }
                };
            }
            Err(e) => return Err(e),
        }
    }

    Ok(next.run(req).await)
}
