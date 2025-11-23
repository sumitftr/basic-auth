use axum::{
    Router,
    routing::{get, post},
};

mod logging;
mod oidc;
mod recovery;
mod register;

#[rustfmt::skip]
pub async fn auth_routes() -> Router {
    Router::new()
        .route("/api/logout_all", post(logging::logout_all))
        .route("/api/logout_devices", post(logging::logout_devices))
        .route("/api/logout", post(logging::logout))
        .layer(axum::middleware::from_fn(crate::middleware::auth_middleware))
        .route("/api/login", post(logging::login))
        .route("/api/oauth2/login", get(oidc::login))
        .route("/api/oauth2/callback", get(oidc::callback))
        .route("/api/forgot_password", post(recovery::forgot_password))
        .route("/api/reset_password", post(recovery::reset_password))
        .route("/api/register/start", post(register::start))
        .route("/api/register/resend_otp", post(register::resend_otp))
        .route("/api/register/verify_email", post(register::verify_email))
        .route("/api/register/set_password", post(register::set_password))
        .route("/api/register/set_username", post(register::set_username))
        .with_state(database::Db::new().await)
}
