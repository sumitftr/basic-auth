use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::{json, response::ErasedJson};
use common::{AppError, session::ActiveSession};
use database::{Db, user::User};
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
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

    let (db_session, active_session, set_cookie_headermap) =
        common::session::create_session(&headers);

    // clearing expired sessions
    common::session::clear_expired_sessions(&mut user.sessions);

    // adding `Session` to primary database
    user.sessions.push(db_session);
    db.update_sessions(&user.username, &user.sessions).await?;

    // activating session by adding it to `Db::active`
    db.make_user_active(active_session, user);

    Ok((
        StatusCode::CREATED,
        set_cookie_headermap,
        json!({
            "message": "Login Successful"
        }),
    ))
}

pub async fn logout(
    State(db): State<Arc<Db>>,
    Extension(active_session): Extension<ActiveSession>,
    Extension(user): Extension<Arc<Mutex<User>>>,
) -> Result<impl IntoResponse, AppError> {
    let (username, sessions) = {
        let mut guard = user.lock().unwrap();
        let i = common::session::get_session_index(&guard.sessions, &active_session)?;
        // deleting the user specified session
        if guard.sessions.len() == 1 {
            guard.sessions.clear();
        } else {
            let tmp_session = guard.sessions.pop().unwrap();
            guard.sessions[i] = tmp_session;
        }
        (guard.username.clone(), guard.sessions.clone())
    };

    // updating sessions list in the primary and in-memory database
    db.update_sessions(&username, &sessions).await?;
    db.remove_active_user(&active_session);

    Ok((
        StatusCode::CREATED,
        active_session.expire(),
        json!({
            "message": "Logout Successful"
        }),
    ))
}

#[derive(serde::Deserialize)]
pub struct LogoutDevicesRequest {
    sessions: Vec<String>,
}

pub async fn logout_devices(
    State(db): State<Arc<Db>>,
    Extension(active_session): Extension<ActiveSession>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(mut body): Json<LogoutDevicesRequest>,
) -> Result<impl IntoResponse, AppError> {
    // cloning username and sessions
    let (username, sessions) = {
        let guard = user.lock().unwrap();
        (guard.username.clone(), guard.sessions.clone())
    };
    // client could send a huge `Vec<String>` which could cause server overhead
    // to neglect that issue this condition is checked
    if sessions.len() < body.sessions.len() {
        return Err(AppError::BadReq("Your selected session list is too long"));
    }
    // removing active session from `body.sessions` if passed
    body.sessions
        .iter_mut()
        .filter(|v| **v != active_session.decrypted_ssid)
        .for_each(|_| ());
    let final_sessions = common::session::delete_selected_sessions(sessions, body.sessions);
    // updating primary and in-memory database with the remaining sessions
    db.update_sessions(&username, &final_sessions).await?;
    user.lock().unwrap().sessions = final_sessions;
    Ok(json!({
        "message": "You sessions has been updated"
    }))
}

pub async fn logout_all(
    State(db): State<Arc<Db>>,
    Extension(active_session): Extension<ActiveSession>,
    Extension(user): Extension<Arc<Mutex<User>>>,
) -> Result<ErasedJson, AppError> {
    // cloning username and sessions
    let (username, mut sessions) = {
        let guard = user.lock().unwrap();
        (guard.username.clone(), guard.sessions.clone())
    };
    // getting session index
    let i = common::session::get_session_index(&sessions, &active_session)?;
    // creating the final sessions vector
    let only_session = {
        if i == sessions.len() - 1 {
            vec![sessions.pop().unwrap()]
        } else {
            let mut tmp = sessions.pop().unwrap();
            std::mem::swap(&mut sessions[i], &mut tmp);
            vec![tmp]
        }
    };
    // updating primary and in-memory database with the only session
    db.update_sessions(&username, &only_session).await?;
    user.lock().unwrap().sessions = only_session;
    Ok(json!({
        "message": "Deleted all other user sessions"
    }))
}
