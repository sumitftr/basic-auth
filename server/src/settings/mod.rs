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
        .route("/api/settings/username", post(username::update_username).get(username::validate_username))
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
    let (u, s) = {
        let guard = user.lock().unwrap();
        (guard.0.clone(), guard.1.clone())
    };

    let sessions: Vec<_> = s
        .iter()
        .map(|session| {
            serde_json::json!({
                "unsigned_ssid": session.unsigned_ssid.to_string(),
                "user_agent": session.user_agent,
                "created_at": session.created_at.to_string(),
                "last_used": session.last_used.to_string(),
            })
        })
        .collect();

    axum_extra::json!({
        "email": u.email,
        "birth_date": u.birth_date.to_string(),
        "username": u.username,
        "display_name": u.display_name,
        "icon": u.icon,
        "banner": u.banner,
        "bio": u.bio,
        "legal_name": u.legal_name,
        "gender": u.gender,
        "phone": u.phone,
        "country": u.country,
        "created": u.created.to_string(),
        "sessions": sessions,
    })
}
