use axum::{routing::get, Router};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/signup", get(signup_page).post(signup))
        .route("/login", get(login_page).post(login))
        .route("/logout", get(logout_page).post(logout))
}

pub async fn login() {}
pub async fn signup() {}
pub async fn logout() {}
pub async fn home() {}
pub async fn login_page() {}
pub async fn signup_page() {}
pub async fn logout_page() {}
