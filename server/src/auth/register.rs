use axum::{
    Json,
    extract::State,
    http::{
        StatusCode,
        header::{self, HeaderMap},
    },
    response::IntoResponse,
};
use common::AppError;
use database::Db;
use serde::Deserialize;
use std::sync::Arc;

/// first step of registering an user

#[derive(Deserialize)]
pub struct CreateUserRequest {
    name: String,
    email: String,
    year: u32,
    month: u8,
    day: u8,
}

pub async fn start(
    State(db): State<Arc<Db>>,
    Json(body): Json<CreateUserRequest>,
) -> Result<String, AppError> {
    // validating user sent data
    let name = common::validation::is_name_valid(&body.name)?;
    common::validation::is_email_valid(&body.email)?;
    let birth_date = common::validation::is_birth_date_valid(body.year, body.month, body.day)?;

    // generating otp
    let otp = common::mail::generate_otp(body.email.as_bytes());
    tracing::info!("OTP: {otp}, Email: {}", body.email);

    // sending otp to the email
    common::mail::send_mail(
        &body.email,
        format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
        format!(
            "Confirm your email address\n {otp}\n Thanks,\n {}",
            &*common::SERVICE_NAME
        ),
    )
    .await?;

    // storing applicant data in memory
    db.create_applicant(name, body.email, birth_date, otp)
        .await?;

    Ok("Your information has been accepted".to_string())
}

#[derive(Deserialize)]
pub struct ResendOtpRequest {
    email: String,
}

pub async fn resend_otp(
    State(db): State<Arc<Db>>,
    Json(body): Json<ResendOtpRequest>,
) -> Result<String, AppError> {
    // generating otp
    let otp = common::mail::generate_otp(body.email.as_bytes());
    db.update_applicant_otp(&body.email, &otp)?;

    // resending otp to the email
    common::mail::send_mail(
        &body.email,
        format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
        format!(
            "Confirm your email address\n {otp}\n Thanks,\n {}",
            &*common::SERVICE_NAME
        ),
    )
    .await?;

    Ok("The email has been sent".to_string())
}

/// second step of registering an user

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    email: String,
    otp: String,
}

pub async fn verify_email(
    State(db): State<Arc<Db>>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<String, AppError> {
    // verifying email by checking if the otp sent by user matches the original one
    db.verify_applicant_email(&body.email, &body.otp).await?;

    // sending email verification success
    common::mail::send_mail(
        &body.email,
        format!("Your email {} has been verified successfully", body.email),
        format!(
            "Your email {} has been verified successfully\n Thanks,\n {}",
            body.email,
            &*common::SERVICE_NAME
        ),
    )
    .await?;

    Ok("Email Verification successful".to_string())
}

/// third step of registering an user

#[derive(Deserialize)]
pub struct SetPasswordRequest {
    email: String,
    password: String,
}

pub async fn set_password(
    State(db): State<Arc<Db>>,
    Json(body): Json<SetPasswordRequest>,
) -> Result<String, AppError> {
    // checking if the user sent password is valid or not
    common::validation::is_password_valid(&body.password)?;

    // setting password in in-memory database
    db.set_applicant_password(&body.email, body.password)?;

    Ok(format!(
        "Your password for email {} has been set",
        body.email
    ))
}

/// last step of registering an user

#[derive(Deserialize)]
pub struct SetUsernameRequest {
    email: String,
    username: String,
}

pub async fn set_username(
    State(db): State<Arc<Db>>,
    headers: HeaderMap,
    Json(body): Json<SetUsernameRequest>,
) -> Result<impl IntoResponse, AppError> {
    // checking validity of the username
    common::validation::is_username_valid(&body.username)?;

    // getting user-agent header
    let user_agent = headers
        .get(header::USER_AGENT)
        .map(|v| v.to_str().unwrap_or_default().to_owned())
        .unwrap_or_default();

    // creating session
    let (user_session, active_user_session, set_cookie_headermap) =
        common::user_session::create_session(user_agent);

    // registering user to primary database
    let user = Arc::clone(&db)
        .set_applicant_username(body.email, body.username.clone(), user_session)
        .await?;

    // activating session by adding it to `Db::active`
    db.make_user_active(active_user_session, user);

    Ok((
        StatusCode::CREATED,
        set_cookie_headermap,
        "User Created".to_string(),
    ))
}
