use axum::{extract::Request, middleware::Next, response::Response};
use common::{
    AppError,
    session::{self, ActiveSession, SessionStatus},
};

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    let active_session = ActiveSession::parse_and_verify_from_headers(req.headers())?;
    let db = database::Db::new().await;

    // Check if user is already in cache (found inside `Db::active`)
    if let Some(user) = db.get_active_user(&active_session) {
        req.extensions_mut().insert(active_session);
        req.extensions_mut().insert(user);
        return Ok(next.run(req).await);
    }

    // User not cached, fetch from database (not found inside `Db::active`)
    let mut user = db.get_user_by_active_session(&active_session).await?;
    let i = session::get_session_index(&user.sessions, &active_session)?;

    match user.sessions[i].session_status() {
        SessionStatus::Valid(_) => {
            // adding session and `User` to `Db::active` for faster access
            let arc_wrapped_user = db.make_user_active(active_session.clone(), user);
            req.extensions_mut().insert(active_session);
            req.extensions_mut().insert(arc_wrapped_user);
        }

        SessionStatus::Expiring(_) | SessionStatus::Refreshable(_) => {
            // automatic session refresh code block
            let (db_session, new_active_session, set_cookie_headermap) =
                session::create_session(req.headers());

            // replacing the old session with new session
            user.sessions[i] = db_session;
            session::clear_expired_sessions(&mut user.sessions);
            db.update_sessions(&user.username, &user.sessions).await?;
            db.make_user_active(new_active_session, user);

            return Err(AppError::RefreshSession(set_cookie_headermap));
        }

        SessionStatus::Invalid => {
            let sessions_len = user.sessions.len();
            session::clear_expired_sessions(&mut user.sessions);

            if user.sessions.len() < sessions_len {
                db.update_sessions(&user.username, &user.sessions).await?;
            }

            return Err(AppError::InvalidSession(active_session.expire()));
        }
    }

    Ok(next.run(req).await)
}
