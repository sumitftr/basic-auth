use axum::http::StatusCode;

pub enum AppError {
    BadReq(&'static str),
    UserNotFound,
    UsernameTaken,
    EmailTaken,
    Auth(&'static str),
    Server(&'static str),
    ServerDefault,
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::BadReq(e) => (StatusCode::BAD_REQUEST, e).into_response(),
            AppError::UserNotFound => (StatusCode::BAD_REQUEST, "User not found").into_response(),
            AppError::UsernameTaken => {
                (StatusCode::CONFLICT, "Username already taken").into_response()
            }
            AppError::EmailTaken => (StatusCode::CONFLICT, "Email already taken").into_response(),
            AppError::Auth(e) => (StatusCode::UNAUTHORIZED, e).into_response(),
            AppError::Server(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
            AppError::ServerDefault => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
        }
    }
}
