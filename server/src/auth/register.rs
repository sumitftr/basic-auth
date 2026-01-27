use crate::ClientSocket;
use axum::extract::ConnectInfo;
use axum::http::{StatusCode, header::HeaderMap};
use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::{json, response::ErasedJson};
use database::Db;
use std::sync::Arc;
use util::AppError;

#[derive(serde::Deserialize)]
pub struct CreateUserRequest {
    name: String,
    email: String,
}

pub async fn start(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    Json(body): Json<CreateUserRequest>,
) -> Result<ErasedJson, AppError> {
    // validating user sent data
    util::validation::is_display_name_valid(&body.name)?;
    util::validation::is_email_valid(&body.email)?;

    let otp = util::generate::otp(&body.email);
    tracing::info!("Email: {}, OTP: {}", body.email, otp);

    db.create_registrant(*conn_info, body.name, body.email.clone(), otp.clone()).await?;

    // sending otp to the email
    util::mail::send(
        body.email,
        format!("{otp} is your {} verification code", &*util::SERVICE_NAME),
        format!("Confirm your email address\n {otp}\n Thanks,\n {}", &*util::SERVICE_NAME),
    )
    .await?;

    Ok(json!({
        "message": "Your information has been accepted"
    }))
}

#[derive(serde::Deserialize)]
pub struct ResendOtpRequest {
    email: String,
}

pub async fn resend_otp(
    State(db): State<Arc<Db>>,
    Json(body): Json<ResendOtpRequest>,
) -> Result<ErasedJson, AppError> {
    let otp = util::generate::otp(&body.email);
    db.update_registrant_otp(&body.email, otp.clone()).await?;

    // resending otp to the email
    util::mail::send(
        body.email,
        format!("{otp} is your {} verification code", &*util::SERVICE_NAME),
        format!("Confirm your email address\n {otp}\n Thanks,\n {}", &*util::SERVICE_NAME),
    )
    .await?;

    Ok(json!({
        "message": "The email has been sent"
    }))
}

#[derive(serde::Deserialize)]
pub struct VerifyEmailRequest {
    email: String,
    otp: String,
}

pub async fn verify_email(
    State(db): State<Arc<Db>>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<ErasedJson, AppError> {
    // verifying email by checking if the otp sent by user matches the original one
    db.verify_registrant_email(&body.email, &body.otp).await?;

    // sending email verification success
    util::mail::send(
        body.email.clone(),
        format!("Your email {} has been verified successfully", body.email),
        format!(
            "Your email {} has been verified successfully\n Thanks,\n {}",
            body.email,
            &*util::SERVICE_NAME
        ),
    )
    .await?;

    Ok(json!({
        "message": "Email Verification successful"
    }))
}

#[derive(serde::Deserialize)]
pub struct SetPasswordRequest {
    email: String,
    password: String,
}

pub async fn set_password(
    State(db): State<Arc<Db>>,
    Json(body): Json<SetPasswordRequest>,
) -> Result<ErasedJson, AppError> {
    util::validation::is_password_strong(&body.password)?;

    db.set_registrant_password(&body.email, body.password).await?;

    Ok(json!({
        "message": format!("Your password for email {} has been set", body.email)
    }))
}

#[derive(serde::Deserialize)]
pub struct SetUsernameRequest {
    email: String,
    username: String,
}

pub async fn set_username(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    headers: HeaderMap,
    Json(body): Json<SetUsernameRequest>,
) -> Result<impl IntoResponse, AppError> {
    util::validation::is_username_valid(&body.username)?;

    // registering user to primary database
    let user = db.set_registrant_username(body.email, body.username).await?;

    let (new_session, _, set_cookie_headermap) =
        util::session::create_session(user.id, &headers, *conn_info);

    let res_body = crate::user_data::arrange(&user, &vec![&new_session]);
    db.add_session(user.id, new_session.clone()).await?;
    db.make_user_active(user, new_session);

    Ok((StatusCode::CREATED, set_cookie_headermap, res_body))
}

#[derive(serde::Deserialize)]
pub struct FinishOidcRequest {
    email: String,
    username: String,
}

pub async fn finish_oidc(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    headers: HeaderMap,
    Json(body): Json<FinishOidcRequest>,
) -> Result<impl IntoResponse, AppError> {
    util::validation::is_username_valid(&body.username)?;

    // registering user to primary database
    let user = db.finish_oidc_application(body.email, body.username).await?;

    let (new_session, _, set_cookie_headermap) =
        util::session::create_session(user.id, &headers, *conn_info);

    let res_body = crate::user_data::arrange(&user, &vec![&new_session]);
    db.add_session(user.id, new_session.clone()).await?;
    db.make_user_active(user, new_session);

    Ok((StatusCode::CREATED, set_cookie_headermap, res_body))
}
