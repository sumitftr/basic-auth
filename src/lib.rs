mod auth;
mod profile;
mod settings;

use axum::Router;

pub fn nest_routes() -> Router {
    let app = Router::new()
        .nest("/", auth::auth_routes())
        .nest("/", profile::profile_routes())
        .nest("/settings", settings::settings_routes());

    return app;
}
