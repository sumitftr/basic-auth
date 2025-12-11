use axum::{
    Json,
    extract::{ConnectInfo, Query, State},
};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::Db;
use std::sync::Arc;

use crate::ClientSocket;

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
    common::validation::is_email_valid(&body.email)?;
    let code = common::generate::hex_64(&body.email);
    db.request_password_reset(*conn_info, &body.email, &code).await?;

    common::mail::send(
        body.email.clone(),
        format!("{} password reset request", &*common::SERVICE_NAME),
        format!(
            "<h1>Reset your password?</h1>\nIf you requested a password reset for {} press on this link {}\nIf you didn't make the request, please ignore this email.\nThanks, {}\n",
            body.email,
            format_args!("{}/api/reset_password?code={code}", &*common::SERVICE_DOMAIN),
            &*common::SERVICE_NAME
        ),
    );

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
    common::validation::is_password_strong(&body.password)?;
    let email = db.reset_password(&q.code, &body.password).await?;

    common::mail::send(
        email.clone(),
        format!("Your {} password has been changed", &*common::SERVICE_NAME),
        format!(
            "Your password for {} has been changed.\nThanks, {}\n",
            email,
            &*common::SERVICE_NAME
        ),
    );

    Ok(json!({
        "message": format!("Your password for {email} has been changed")
    }))
}
