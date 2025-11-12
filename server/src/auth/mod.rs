use axum::{Router, routing::post};

mod recovery;
mod register;
mod session;

#[rustfmt::skip]
pub async fn auth_routes() -> Router {
    Router::new()
        .route("/api/user/logout_all", post(session::logout_all))
        .route("/api/user/logout_devices", post(session::logout_devices))
        .route("/api/user/logout", post(session::logout))
        .layer(axum::middleware::from_fn(crate::middleware::auth_middleware))
        .route("/api/user/login", post(session::login))
        .route("/api/forgot_password", post(recovery::forgot_password))
        .route("/api/reset_password", post(recovery::reset_password))
        .route("/api/user/register/start", post(register::start))
        .route("/api/user/register/resend_otp", post(register::resend_otp))
        .route("/api/user/register/verify_email", post(register::verify_email))
        .route("/api/user/register/set_password", post(register::set_password))
        .route("/api/user/register/set_username", post(register::set_username))
        .with_state(database::Db::new().await)
}
