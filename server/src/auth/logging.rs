use axum::{
    Extension, Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_extra::{json, response::ErasedJson};
use database::{Db, UserData};
use std::sync::Arc;
use util::{AppError, session::ParsedSession};

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    email: Option<String>,
    username: Option<String>,
    password: String,
}

pub async fn login(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<crate::ClientSocket>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = match (&body.email, &body.username) {
        (Some(email), None) => db.authenticate_user_by_email(email, &body.password).await?,
        (None, Some(username)) => db.authenticate_user_by_username(username, &body.password).await?,
        (Some(_), Some(_)) => return Err(AppError::BadReq("Either email or username is allowed")),
        (None, None) => return Err(AppError::BadReq("No email or username found")),
    };

    let (new_session, parsed_session, set_cookie_headermap) =
        util::session::create_session(user.id, &headers, *conn_info);
    let res_body = crate::user_data::arrange(&user, &vec![&new_session]);

    // adding `Session` to primary database
    db.add_session(user.id, new_session.clone()).await?;

    // activating session by adding it to `Db::active`
    if let Some((arc_wrapped, is_session_present)) = db.get_active_user(&parsed_session)
        && !is_session_present
    {
        arc_wrapped.lock().unwrap().1.push(new_session);
    } else {
        db.make_user_active(user, new_session);
    }

    Ok((StatusCode::CREATED, set_cookie_headermap, res_body))
}

pub async fn logout(
    State(db): State<Arc<Db>>,
    Extension(parsed_session): Extension<ParsedSession>,
    Extension(user): Extension<UserData>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = user.lock().unwrap().0.id;

    db.remove_session(user_id, parsed_session.unsigned_ssid).await?;
    db.remove_active_user(&parsed_session);

    Ok((
        StatusCode::CREATED,
        util::session::expire_session(),
        json!({
            "message": "Logout Successful"
        }),
    ))
}

#[derive(serde::Deserialize)]
pub struct LogoutDevicesRequest {
    sessions: Vec<String>,
    password: String,
}

pub async fn logout_devices(
    State(db): State<Arc<Db>>,
    Extension(parsed_session): Extension<ParsedSession>,
    Extension(user): Extension<UserData>,
    Json(body): Json<LogoutDevicesRequest>,
) -> Result<impl IntoResponse, AppError> {
    let (user_id, mut session_list) = {
        let guard = user.lock().unwrap();
        if guard.0.password.as_ref().is_none_or(|v| v != &body.password) {
            return Err(AppError::PasswordMismatch);
        }
        (guard.0.id, guard.1.clone())
    };

    let mut mapped_unsigned_ssids = vec![];
    for device in body.sessions {
        let uid = match uuid::Uuid::try_from(device) {
            Ok(v) => v,
            Err(_) => return Err(AppError::InvalidData("Invalid Session found")),
        };
        if uid != parsed_session.unsigned_ssid {
            mapped_unsigned_ssids.push(uid);
            session_list.retain(|s| uid != s.unsigned_ssid);
        }
    }

    // updating primary and in-memory database with the only session
    db.remove_selected_sessions(user_id, &mapped_unsigned_ssids).await.unwrap();
    user.lock().unwrap().1 = session_list;

    Ok(json!({
        "message": "Your sessions has been updated"
    }))
}

#[derive(serde::Deserialize)]
pub struct LogoutAllRequest {
    password: String,
}

pub async fn logout_all(
    State(db): State<Arc<Db>>,
    Extension(parsed_session): Extension<ParsedSession>,
    Extension(user): Extension<UserData>,
    Json(body): Json<LogoutAllRequest>,
) -> Result<ErasedJson, AppError> {
    let (user_id, mut session_list) = {
        let guard = user.lock().unwrap();
        if guard.0.password.as_ref().is_none_or(|v| v != &body.password) {
            return Err(AppError::PasswordMismatch);
        }
        (guard.0.id, guard.1.clone())
    };

    // deleting all other sessions except the current one
    session_list.retain(|v| v.unsigned_ssid == parsed_session.unsigned_ssid);

    // updating primary and in-memory database with the only session
    db.remove_all_sessions(user_id, parsed_session.unsigned_ssid).await?;
    user.lock().unwrap().1 = session_list;

    Ok(json!({
        "message": "Your all other sessions has been deleted"
    }))
}
