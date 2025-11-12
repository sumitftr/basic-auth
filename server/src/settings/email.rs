use axum::{Extension, Json, extract::State};
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
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<UpdateEmailRequest>,
) -> Result<String, AppError> {
    let email = user.lock().unwrap().email.clone();
    // checking whether the new email is same as original email or not
    if email == body.new_email {
        return Err(AppError::BadReq(
            "Your new email cannot be same as of your original email",
        ));
    }
    // checking if the new email is valid or not
    common::validation::is_email_valid(&body.new_email)?;
    // checking if the new email is available or not
    db.is_email_available(&body.new_email).await?;
    // generating otp
    let otp = common::mail::generate_otp(body.new_email.as_bytes());
    // sending mail to the new email for verification
    common::mail::send_mail(
        &body.new_email,
        format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
        format!(
            "Confirm your email address\n {otp}\n Thanks,\n {}",
            &*common::SERVICE_NAME
        ),
    )
    .await?;
    // adding an entry to in-memory cache for further checking
    db.add_verification_entry(email, body.new_email, otp.clone());
    Ok(otp)
}

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    otp: String,
}

pub async fn verify_email(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<String, AppError> {
    let old_email = user.lock().unwrap().email.clone();
    if let Some((new_email, otp)) = db.get_verification_entry(&old_email) {
        if otp == body.otp {
            db.update_email(&old_email, &new_email).await?;
            db.remove_verification_entry(&old_email);
            user.lock().unwrap().email = new_email;
            Ok("Your email has been verified".to_string())
        } else {
            Err(AppError::BadReq("Invalid OTP"))
        }
    } else {
        Err(AppError::ServerError)
    }
}
