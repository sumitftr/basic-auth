use crate::ClientSocket;
use axum::extract::ConnectInfo;
use axum::http::{StatusCode, header::HeaderMap};
use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::Db;
use std::sync::Arc;
use time::UtcOffset;

#[derive(serde::Deserialize)]
pub struct CreateUserRequest {
    name: String,
    email: String,
    year: u32,
    month: u8,
    day: u8,
    offset_hours: i8,
    offset_minutes: i8,
    offset_seconds: i8,
}

pub async fn start(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    Json(body): Json<CreateUserRequest>,
) -> Result<ErasedJson, AppError> {
    // validating user sent data
    common::validation::is_display_name_valid(&body.name)?;
    common::validation::is_email_valid(&body.email)?;
    let offset = UtcOffset::from_hms(body.offset_hours, body.offset_minutes, body.offset_seconds)
        .map_err(|_| AppError::InvalidData("Invalid UTC Offset"))?;
    let birth_date =
        common::validation::is_birth_date_valid(body.year, body.month, body.day, offset)?;

    let otp = common::generate::otp(&body.email);
    tracing::info!("Email: {}, OTP: {}", body.email, otp);

    db.create_registrant(*conn_info, body.name, body.email.clone(), birth_date, otp.clone()).await?;

    // sending otp to the email
    common::mail::send(
        body.email,
        format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
        format!("Confirm your email address\n {otp}\n Thanks,\n {}", &*common::SERVICE_NAME),
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
    let otp = common::generate::otp(&body.email);
    db.update_registrant_otp(&body.email, otp.clone()).await?;

    // resending otp to the email
    common::mail::send(
        body.email,
        format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
        format!("Confirm your email address\n {otp}\n Thanks,\n {}", &*common::SERVICE_NAME),
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
    common::mail::send(
        body.email.clone(),
        format!("Your email {} has been verified successfully", body.email),
        format!(
            "Your email {} has been verified successfully\n Thanks,\n {}",
            body.email,
            &*common::SERVICE_NAME
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
    common::validation::is_password_strong(&body.password)?;

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
    common::validation::is_username_valid(&body.username)?;

    // registering user to primary database
    let user = db.set_registrant_username(body.email, body.username).await?;

    let (new_session, _, set_cookie_headermap) =
        common::session::create_session(user.id, &headers, *conn_info);

    let res_body = crate::user_data::arrange(&user, &vec![&new_session]);
    db.add_session(user.id, new_session.clone()).await?;
    db.make_user_active(user, new_session);

    Ok((StatusCode::CREATED, set_cookie_headermap, res_body))
}

#[derive(serde::Deserialize)]
pub struct FinishOidcRequest {
    email: String,
    year: u32,
    month: u8,
    day: u8,
    offset_hours: i8,
    offset_minutes: i8,
    offset_seconds: i8,
    username: String,
}

pub async fn finish_oidc(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    headers: HeaderMap,
    Json(body): Json<FinishOidcRequest>,
) -> Result<impl IntoResponse, AppError> {
    common::validation::is_username_valid(&body.username)?;

    let offset = UtcOffset::from_hms(body.offset_hours, body.offset_minutes, body.offset_seconds)
        .map_err(|_| AppError::InvalidData("Invalid UTC Offset"))?;
    let birth_date =
        common::validation::is_birth_date_valid(body.year, body.month, body.day, offset)?;

    // registering user to primary database
    let user = db.finish_oidc_application(body.email, birth_date, body.username).await?;

    let (new_session, _, set_cookie_headermap) =
        common::session::create_session(user.id, &headers, *conn_info);

    let res_body = crate::user_data::arrange(&user, &vec![&new_session]);
    db.add_session(user.id, new_session.clone()).await?;
    db.make_user_active(user, new_session);

    Ok((StatusCode::CREATED, set_cookie_headermap, res_body))
}
