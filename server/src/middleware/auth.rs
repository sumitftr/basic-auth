use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::{IntoResponse, Response},
};
use common::{
    AppError,
    session::{ParsedSession, SessionStatus},
};

pub async fn auth_middleware(
    ConnectInfo(conn_info): ConnectInfo<crate::ClientSocket>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let parsed_session = ParsedSession::parse_and_verify_from_headers(req.headers())?;
    let db = database::Db::new().await;

    // Check if user is already in cache (found inside `Db::active`)
    if let Some((arc_wrapped, is_session_present)) = db.get_active_user(&parsed_session) {
        // if the session is not found in cache
        if !is_session_present {
            let session = db.get_session(&parsed_session).await?;
            let mut guard = arc_wrapped.lock().unwrap();
            guard.1.push(session);
        }
        req.extensions_mut().insert(parsed_session);
        req.extensions_mut().insert(arc_wrapped);
        return Ok(next.run(req).await);
    }

    // User not cached, fetch from database (not found inside `Db::active`)
    let (user, session) = db.get_all_by_parsed_session(&parsed_session).await?;

    match session.session_status() {
        SessionStatus::Valid(_) => {
            // adding session and `User` to `Db::active`
            let arc_wrapped = db.make_user_active(user, session);
            req.extensions_mut().insert(parsed_session);
            req.extensions_mut().insert(arc_wrapped);
        }

        SessionStatus::Expiring(_) | SessionStatus::Refreshable(_) => {
            // automatic session refresh code block
            let (new_session, _, set_cookie_headermap) =
                common::session::create_session(user.id, req.headers(), *conn_info);

            // replacing the old session with new session
            db.add_session(user.id, new_session.clone()).await?;
            db.remove_session(user.id, session.unsigned_ssid).await?;
            db.make_user_active(user, new_session);

            // in the case of `Expiring` the new ssid will override the old one
            return Ok(set_cookie_headermap.into_response());
        }

        SessionStatus::Invalid => {
            db.clear_expired_sessions(user.id).await?;

            return Err(AppError::InvalidSession(common::session::expire_session()));
        }
    }

    Ok(next.run(req).await)
}
