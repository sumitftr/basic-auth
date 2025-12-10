use axum::http::{StatusCode, header::HeaderMap};
use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::Db;
use std::sync::Arc;

/// first step of registering an user

#[derive(serde::Deserialize)]
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
) -> Result<ErasedJson, AppError> {
    // validating user sent data
    common::validation::is_display_name_valid(&body.name)?;
    common::validation::is_email_valid(&body.email)?;
    let birth_date = common::validation::is_birth_date_valid(body.year, body.month, body.day)?;

    // generating otp
    let otp = common::generate::otp(&body.email);

    // storing applicant data in memory
    db.create_applicant(body.name, body.email.clone(), birth_date, otp.clone())
        .await?;

    // sending otp to the email
    common::mail::send(
        body.email,
        format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
        format!("Confirm your email address\n {otp}\n Thanks,\n {}", &*common::SERVICE_NAME),
    );

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
    // generating otp
    let otp = common::generate::otp(&body.email);
    db.update_applicant_otp(&body.email, &otp).await?;

    // resending otp to the email
    common::mail::send(
        body.email,
        format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
        format!("Confirm your email address\n {otp}\n Thanks,\n {}", &*common::SERVICE_NAME),
    );

    Ok(json!({
        "message": "The email has been sent"
    }))
}

/// second step of registering an user

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
    );

    Ok(json!({
        "message": "Email Verification successful"
    }))
}

/// third step of registering an user

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
    db.set_applicant_password(&body.email, &body.password).await?;

    Ok(json!({
        "message": format!("Your password for email {} has been set", body.email)
    }))
}

/// last step of registering an user

#[derive(serde::Deserialize)]
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

    // creating session
    let (db_session, active_session, set_cookie_headermap) =
        common::session::create_session(&headers);

    // registering user to primary database
    let user = Arc::clone(&db)
        .set_applicant_username(body.email, body.username, db_session)
        .await?;

    // activating session by adding it to `Db::active`
    db.make_user_active(active_session, user);

    Ok((
        StatusCode::CREATED,
        set_cookie_headermap,
        json!({
            "message": "User Created"
        }),
    ))
}

/// for finishing oidc application

#[derive(serde::Deserialize)]
pub struct FinishOidcRequest {
    email: String,
    year: u32,
    month: u8,
    day: u8,
    username: String,
}

pub async fn finish_oidc(
    State(db): State<Arc<Db>>,
    headers: HeaderMap,
    Json(body): Json<FinishOidcRequest>,
) -> Result<impl IntoResponse, AppError> {
    // checking validity of the username
    let birth_date = common::validation::is_birth_date_valid(body.year, body.month, body.day)?;
    common::validation::is_username_valid(&body.username)?;

    // creating session
    let (db_session, active_session, set_cookie_headermap) =
        common::session::create_session(&headers);

    // registering user to primary database
    let user = Arc::clone(&db)
        .finish_oidc_application(body.email, birth_date, body.username, db_session)
        .await?;

    // activating session by adding it to `Db::active`
    db.make_user_active(active_session, user);

    Ok((
        StatusCode::CREATED,
        set_cookie_headermap,
        json!({
            "message": "User Created"
        }),
    ))
}
