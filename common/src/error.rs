use axum::http::{HeaderMap, StatusCode};

#[derive(Debug)]
pub enum AppError {
    BadReq(&'static str),
    UserNotFound,
    UsernameTaken,
    EmailTaken,
    Auth(&'static str),
    InvalidSession(HeaderMap),
    RefreshSession(HeaderMap),
    Server(Box<dyn std::error::Error>),
    ServerDefault,
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BadReq(e) => (StatusCode::BAD_REQUEST, e).into_response(),
            Self::UserNotFound => (StatusCode::BAD_REQUEST, "User not found").into_response(),
            Self::UsernameTaken => (StatusCode::CONFLICT, "Username already taken").into_response(),
            Self::EmailTaken => (StatusCode::CONFLICT, "Email already taken").into_response(),
            Self::Auth(e) => (StatusCode::UNAUTHORIZED, e).into_response(),
            Self::InvalidSession(h) => {
                (StatusCode::UNAUTHORIZED, h, "Invalid Session").into_response()
            }
            Self::RefreshSession(h) => (StatusCode::OK, h, "Session Refreshed").into_response(),
            Self::Server(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            Self::ServerDefault => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
        }
    }
}
