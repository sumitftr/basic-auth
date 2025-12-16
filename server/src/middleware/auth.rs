use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use common::{
    AppError,
    session::{ParsedSession, SessionStatus},
};

pub async fn auth_middleware(
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let parsed_session = ParsedSession::parse_and_verify_from_headers(req.headers())?;
    let db = database::Db::new().await;

    // Check if user is already in cache (found inside `Db::active`)
    if let Some((arc_wrapped, is_present)) = db.get_active_user(&parsed_session) {
        // if the session is not found in cache
        if !is_present {
            db.make_session_active(arc_wrapped, parsed_session).await?;
        }
        req.extensions_mut().insert(parsed_session);
        req.extensions_mut().insert(arc_wrapped);
        return Ok(next.run(req).await);
    }

    // User not cached, fetch from database (not found inside `Db::active`)
    let (mut user, session) = db.get_user_by_parsed_session(&parsed_session).await?;

    match session.session_status() {
        SessionStatus::Valid(_) => {
            // adding session and `User` to `Db::active` for faster access
            let arc_wrapped = db.make_user_active(user, session);
            req.extensions_mut().insert(parsed_session);
            req.extensions_mut().insert(arc_wrapped);
        }

        SessionStatus::Expiring(_) | SessionStatus::Refreshable(_) => {
            // automatic session refresh code block
            let (db_session, new_parsed_session, set_cookie_headermap) =
                common::session::create_session(user.id, req.headers(), conn_info);

            // replacing the old session with new session
            user.sessions[i] = db_session;
            common::session::clear_expired_sessions(&mut user.sessions);
            db.update_sessions(&user.username, &user.sessions).await?;
            db.make_user_active(new_parsed_session, user);

            // in the case of `Expiring` the new ssid will override the old one
            return Ok(set_cookie_headermap.into_response());
        }

        SessionStatus::Invalid => {
            let sessions_len = user.sessions.len();
            common::session::clear_expired_sessions(&mut user.sessions);

            if user.sessions.len() < sessions_len {
                db.update_sessions(&user.username, &user.sessions).await?;
            }

            return Err(AppError::InvalidSession(common::session::expire_session()));
        }
    }

    Ok(next.run(req).await)
}
