use axum::{Router, routing::post};

mod register;
mod session;

#[rustfmt::skip]
pub async fn auth_routes() -> Router {
    Router::new()
        .route("/api/user/register/start", post(register::start))
        .route("/api/user/register/resend_otp", post(register::resend_otp))
        .route("/api/user/register/verify_email", post(register::verify_email))
        .route("/api/user/register/set_password", post(register::set_password))
        .route("/api/user/register/set_username", post(register::set_username))
        .route("/api/user/login", post(session::login))
        .route("/api/user/logout", post(session::logout))
        .with_state(database::Db::new().await)
}
