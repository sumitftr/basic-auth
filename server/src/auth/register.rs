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

    // storing applicant data in memory
    db.create_applicant(*conn_info, body.name, body.email.clone(), birth_date, otp.clone()).await?;

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
    db.update_applicant_otp(&body.email, otp.clone()).await?;

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
    db.verify_applicant_email(&body.email, &body.otp).await?;

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
    // checking if the user sent password is valid or not
    common::validation::is_password_strong(&body.password)?;

    // setting password in in-memory database
    db.set_applicant_password(&body.email, body.password).await?;

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
    // checking validity of the username
    common::validation::is_username_valid(&body.username)?;

    // registering user to primary database
    let user = db.set_applicant_username(body.email, body.username).await?;

    let (new_session, _, set_cookie_headermap) =
        common::session::create_session(user.id, &headers, *conn_info);

    // adding `Session` to primary database
    db.add_session(new_session.clone()).await?;

    // activating session by adding it to `Db::active`
    db.make_user_active(user, new_session);

    Ok((
        StatusCode::CREATED,
        set_cookie_headermap,
        json!({
            "message": "User Created"
        }),
    ))
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
    // checking validity of the username
    let offset = UtcOffset::from_hms(body.offset_hours, body.offset_minutes, body.offset_seconds)
        .map_err(|_| AppError::InvalidData("Invalid UTC Offset"))?;
    let birth_date =
        common::validation::is_birth_date_valid(body.year, body.month, body.day, offset)?;
    common::validation::is_username_valid(&body.username)?;

    // registering user to primary database
    let user = db.finish_oidc_application(body.email, birth_date, body.username).await?;

    let (new_session, _, set_cookie_headermap) =
        common::session::create_session(user.id, &headers, *conn_info);

    // adding `Session` to primary database
    db.add_session(new_session.clone()).await?;

    // activating session by adding it to `Db::active`
    db.make_user_active(user, new_session);

    Ok((
        StatusCode::CREATED,
        set_cookie_headermap,
        json!({
            "message": "User Created"
        }),
    ))
}
