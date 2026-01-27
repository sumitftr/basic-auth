use crate::ClientSocket;
use axum::{
    Extension, Json,
    extract::{ConnectInfo, State},
};
use axum_extra::{json, response::ErasedJson};
use database::{Db, UserData};
use serde::Deserialize;
use std::sync::Arc;
use util::{AppError, oauth::OAuthProvider};

#[derive(Deserialize)]
pub struct UpdateEmailRequest {
    new_email: String,
    password: String,
}

pub async fn update_email(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    Extension(user): Extension<UserData>,
    Json(body): Json<UpdateEmailRequest>,
) -> Result<ErasedJson, AppError> {
    let email = {
        let guard = user.lock().unwrap();
        if guard.0.password.as_ref().is_none_or(|v| v != &body.password) {
            return Err(AppError::PasswordMismatch);
        }
        guard.0.email.clone()
    };
    // checking whether the new email is same as original email or not
    if email == body.new_email {
        return Err(AppError::BadReq("Your new email cannot be same as of your original email"));
    }
    util::validation::is_email_valid(&body.new_email)?;

    let otp = util::generate::otp(&body.new_email);
    tracing::info!("Email: {}, OTP: {}", email, otp);

    // adding an entry to database for further checking
    db.request_email_update(*conn_info, email, body.new_email.clone(), otp.clone()).await?;

    // sending mail to the new email for verification
    util::mail::send(
        body.new_email,
        format!("{otp} is your {} verification code", &*util::SERVICE_NAME),
        format!("Confirm your email address\n {otp}\n Thanks,\n {}", &*util::SERVICE_NAME),
    )
    .await?;

    Ok(json!({
        "message": "Please verify your email",
    }))
}

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    new_email: String,
    otp: String,
}

pub async fn verify_email(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserData>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<ErasedJson, AppError> {
    let old_email = user.lock().unwrap().0.email.clone();
    db.update_email(&old_email, body.new_email.clone(), &body.otp).await?;
    user.lock().unwrap().0.email = body.new_email.clone();
    Ok(json!({
        "email": body.new_email,
        "message": "Your email has been verified",
    }))
}

pub async fn connect_email(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserData>,
) -> Result<ErasedJson, AppError> {
    let email = user.lock().unwrap().0.email.clone();
    let domain = unsafe { email.split('@').next_back().unwrap_unchecked() };
    let provider = OAuthProvider::from_domain(domain);
    match provider {
        OAuthProvider::Google => {
            db.update_oauth_provider(&email, provider).await?;
            user.lock().unwrap().0.oauth_provider = provider;
            Ok(json!({
                "oauth_provider": provider.get_str(),
                "message": format!("Your email is now connected with {}", provider.get_str()),
            }))
        }
        OAuthProvider::None => Err(AppError::BadReq("Unsupported OAuth Provider")),
    }
}
