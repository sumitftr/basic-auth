use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use common::{AppError, user_session::ActiveUserSession};
use database::{Db, user::User};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct LoginRequest {
    email: Option<String>,
    username: Option<String>,
    password: String,
}

pub async fn login(
    State(db): State<Arc<Db>>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.email.is_some() && body.username.is_some() {
        return Err(AppError::BadReq("Either email or username is allowed"));
    }

    if body.email.is_none() && body.username.is_none() {
        return Err(AppError::BadReq("No email or username found"));
    }

    // authenticating user by password
    let mut user = if let Some(email) = body.email {
        db.authenticate_user_by_email(&email, &body.password)
            .await?
    } else if let Some(username) = body.username {
        db.authenticate_user_by_username(&username, &body.password)
            .await?
    } else {
        // this is an unreachable statement
        return Err(AppError::ServerError);
    };

    // creating session for user
    let user_agent = headers
        .get(header::USER_AGENT)
        .map(|v| v.to_str().unwrap_or_default().to_owned())
        .unwrap_or_default();
    let (user_session, active_user_session, set_cookie_headermap) =
        common::user_session::create_session(user_agent);

    // clearing expired sessions
    common::user_session::clear_expired_sessions(&mut user.sessions);

    // adding `UserSession` to primary database
    user.sessions.push(user_session);
    db.update_sessions(&user.username, &user.sessions).await?;

    // activating session by adding it to `Db::active`
    db.make_user_active(active_user_session, user);

    Ok((
        StatusCode::CREATED,
        set_cookie_headermap,
        "Login Successful".to_string(),
    ))
}

pub async fn logout(
    State(db): State<Arc<Db>>,
    Extension(active_user_session): Extension<ActiveUserSession>,
    Extension(mut user): Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    let decrypted_ssid = active_user_session.verify()?;
    let i = common::user_session::get_session_index(&user.sessions, decrypted_ssid)?;
    // deleting the user specified session
    if user.sessions.len() == 1 {
        user.sessions.clear();
    } else {
        let tmp_session = user.sessions.pop().unwrap();
        user.sessions[i] = tmp_session;
    }

    // updating sessions list in the primary database
    db.update_sessions(&user.username, &user.sessions).await?;

    // removing the user from `DB::active`
    db.remove_active_user(&active_user_session);

    Ok((
        StatusCode::CREATED,
        active_user_session.expire(),
        "Logout Successful".to_string(),
    ))
}
