use crate::ClientSocket;
use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
};
use axum_extra::{json, response::ErasedJson};
use database::Db;
use std::sync::Arc;
use util::AppError;

#[derive(serde::Deserialize)]
pub struct ForgotPasswordRequest {
    email: String,
}

// improve this route such that it can reset password using username and phone also
pub async fn forgot_password(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    Json(body): Json<ForgotPasswordRequest>,
) -> Result<ErasedJson, AppError> {
    util::validation::is_email_valid(&body.email)?;
    let code = util::generate::hex_64(&body.email);
    db.request_password_reset(*conn_info, body.email.clone(), code.clone());

    util::mail::send(
        body.email.clone(),
        format!("{} password reset request", &*util::SERVICE_NAME),
        format!(
            "<h1>Reset your password?</h1>\nIf you requested a password reset for {} press on this link {}\nIf you didn't make the request, please ignore this email.\nThanks, {}\n",
            body.email,
            format_args!("{}/api/reset_password?code={code}", &*util::SERVICE_DOMAIN),
            &*util::SERVICE_NAME
        ),
    ).await?;

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
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    Query(q): Query<ResetPasswordQuery>,
    Json(body): Json<ResetPasswordRequest>,
) -> Result<ErasedJson, AppError> {
    util::validation::is_password_strong(&body.password)?;
    let email = db.reset_password(*conn_info, &q.code, &body.password).await?;

    util::mail::send(
        email.clone(),
        format!("Your {} password has been changed", &*util::SERVICE_NAME),
        format!("Your password for {} has been changed.\nThanks, {}\n", email, &*util::SERVICE_NAME),
    )
    .await?;

    Ok(json!({
        "message": format!("Your password for {email} has been changed")
    }))
}
