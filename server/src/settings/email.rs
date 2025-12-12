use crate::ClientSocket;
use axum::{
    Extension, Json,
    extract::{ConnectInfo, State},
};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::{Db, user::User};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct UpdateEmailRequest {
    new_email: String,
}

pub async fn update_email(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<UpdateEmailRequest>,
) -> Result<ErasedJson, AppError> {
    let email = user.lock().unwrap().email.clone();
    // checking whether the new email is same as original email or not
    if email == body.new_email {
        return Err(AppError::BadReq("Your new email cannot be same as of your original email"));
    }

    common::validation::is_email_valid(&body.new_email)?;
    let otp = common::generate::otp(&body.new_email);

    // adding an entry to database for further checking
    db.request_email_update(*conn_info, email, body.new_email.clone(), otp.clone())
        .await?;

    // sending mail to the new email for verification
    common::mail::send(
        body.new_email,
        format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
        format!("Confirm your email address\n {otp}\n Thanks,\n {}", &*common::SERVICE_NAME),
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
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<ErasedJson, AppError> {
    let old_email = user.lock().unwrap().email.clone();
    db.update_email(&old_email, body.new_email.clone(), &body.otp).await?;
    user.lock().unwrap().email = body.new_email.clone();
    Ok(json!({
        "email": body.new_email,
        "message": "Your email has been verified",
    }))
}
