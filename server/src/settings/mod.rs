use axum::routing::{get, post};

mod account;
mod email;
mod metadata;
mod password;
mod phone;
mod username;

#[rustfmt::skip]
pub async fn settings_routes() -> axum::Router {
    axum::Router::new()
        .route("/api/settings", get(fetch_settings))
        .route("/api/settings/email", post(email::update_email))
        .route("/api/settings/verify_email", post(email::verify_email))
        .route("/api/settings/username", post(username::update_username))
        .route("/api/settings/password", post(password::update_password))
        .route("/api/settings/legal_name", post(metadata::update_legal_name))
        .route("/api/settings/birth_date", post(metadata::update_birth_date))
        .route("/api/settings/gender", post(metadata::update_gender))
        .route("/api/settings/phone", post(phone::update_phone))
        .route("/api/settings/verify_phone", post(phone::verify_phone))
        .route("/api/settings/country", post(metadata::update_country))
        .route("/api/settings/delete_account", post(account::delete_account))
        .layer(axum::middleware::from_fn(crate::middleware::auth_middleware))
        .with_state(database::Db::new().await)
}

pub async fn fetch_settings(
    axum::Extension(user): axum::Extension<database::UserInfo>,
) -> axum_extra::response::ErasedJson {
    let res = user.lock().unwrap().0.clone();
    axum_extra::json!({
        "email": res.email,
        "birth_date": res.birth_date.to_string(),
        "username": res.username,
        "display_name": res.display_name,
        "icon": res.icon,
        "banner": res.banner,
        "bio": res.bio,
        "legal_name": res.legal_name,
        "gender": res.gender,
        "phone": res.phone,
        "country": res.country,
        "created": res.created.to_string(),
    })
}
