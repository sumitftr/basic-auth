use axum::{Router, routing::post};
use database::Db;
use std::sync::Arc;

mod login;
mod register;

pub(super) fn auth_routes(db: Arc<Db>) -> Router {
    Router::new()
        .route("/api/register/create_user", post(register::create_user))
        .route("/api/register/resend_otp", post(register::resend_otp))
        .route("/api/register/verify_email", post(register::verify_email))
        .route("/api/register/set_password", post(register::set_password))
        .route("/api/register/set_username", post(register::set_username))
        .route("/api/user/login", post(login::login))
        .with_state(db)
}
