use axum::http::{HeaderMap, StatusCode};

#[derive(Debug)]
pub enum AppError {
    BadReq(&'static str),
    InvalidEmailFormat,
    UserNotFound,
    UsernameTaken,
    EmailTaken,
    WrongPassword,
    AuthError(&'static str),
    InvalidSession(HeaderMap),
    RefreshSession(HeaderMap),
    ServerError,
}

#[rustfmt::skip]
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BadReq(e) => {
                (StatusCode::BAD_REQUEST, e).into_response()
            }
            Self::InvalidEmailFormat => {
                (StatusCode::BAD_REQUEST, "Invalid Email Format").into_response()
            }
            Self::UserNotFound => {
                (StatusCode::NOT_FOUND, "User not found").into_response()
            }
            Self::UsernameTaken => {
                (StatusCode::CONFLICT, "Username already taken").into_response()
            }
            Self::EmailTaken => {
                (StatusCode::CONFLICT, "Email already taken").into_response() 
            }
            Self::WrongPassword => {
                (StatusCode::UNAUTHORIZED, "Password didn't match").into_response()
            }
            Self::AuthError(e) => {
                (StatusCode::UNAUTHORIZED, e).into_response()
            }
            Self::InvalidSession(set_cookies) => {
                (StatusCode::UNAUTHORIZED, set_cookies, "Invalid Session").into_response()
            }
            Self::RefreshSession(set_cookies) => {
                (StatusCode::OK, set_cookies, "Session Refreshed").into_response()
            }
            Self::ServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
        }
    }
}
