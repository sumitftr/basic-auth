use axum::{
    Json,
    extract::{Query, State},
};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::Db;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct ForgotPasswordRequest {
    email: String,
}

// improve this route such that it can reset password using username and phone also
pub async fn forgot_password(
    State(db): State<Arc<Db>>,
    Json(body): Json<ForgotPasswordRequest>,
) -> Result<ErasedJson, AppError> {
    let code = common::generate::hex_64(&body.email);
    db.add_recovery_entry(code.clone(), body.email.clone());

    common::mail::send(
        body.email.as_str(),
        format!("{} password reset request", &*common::SERVICE_NAME),
        format!(
            "<h1>Reset your password?</h1>\nIf you requested a password reset for {} press on this link {}\nIf you didn't make the request, please ignore this email.\nThanks, {}\n",
            body.email,
            format_args!("{}/api/reset_password?code={code}", &*common::SERVICE_DOMAIN),
            &*common::SERVICE_NAME
        ),
    )
    .await?;

    Ok(json!({
        "message": format!("Check your email to reset password")
    }))
}

#[derive(serde::Deserialize)]
pub struct ResetPasswordQuery {
    code: String,
}

#[derive(serde::Deserialize)]
pub struct ResetPasswordRequest {
    password: String,
}

pub async fn reset_password(
    State(db): State<Arc<Db>>,
    Query(q): Query<ResetPasswordQuery>,
    Json(body): Json<ResetPasswordRequest>,
) -> Result<ErasedJson, AppError> {
    if let Some(email) = db.get_recovery_entry(&q.code) {
        common::validation::is_password_valid(&body.password)?;
        db.update_password(&email, &body.password).await?;
        db.remove_recovery_entry(&email);

        common::mail::send(
            &email,
            format!("Your {} password has been changed", &*common::SERVICE_NAME),
            format!(
                "Your password for {} has been changed.\nThanks, {}\n",
                email,
                &*common::SERVICE_NAME
            ),
        )
        .await?;

        Ok(json!({
            "message": format!("Your password for {email} has been changed")
        }))
    } else {
        Err(AppError::BadReq("Your password reset link has expired"))
    }
}
