use axum::http::{HeaderMap, StatusCode};

#[derive(PartialEq, Debug)]
pub enum AppError {
    BadReq(&'static str),
    Unauthorized(&'static str),
    InvalidData(&'static str),
    InvalidDataFmt(String),
    InvalidEmailFormat,
    UserNotFound,
    UsernameTaken,
    EmailTaken,
    WrongPassword,
    SessionExpired,
    InvalidSession(HeaderMap),
    RefreshSession(HeaderMap),
    ServerError,
}

#[rustfmt::skip]
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BadReq(e) => {
                (StatusCode::BAD_REQUEST, JsonMsg::new(e)).into_response()
            }
            Self::Unauthorized(e) => {
                (StatusCode::UNAUTHORIZED, JsonMsg::new(e)).into_response()
            }
            Self::InvalidData(e) => {
                (StatusCode::BAD_REQUEST, JsonMsg::new(e)).into_response()
            }
            Self::InvalidDataFmt(e) => {
                (StatusCode::BAD_REQUEST, JsonMsg::new(&e)).into_response()
            }
            Self::InvalidEmailFormat => {
                (StatusCode::BAD_REQUEST, JsonMsg::new("Invalid Email Format")).into_response()
            }
            Self::UserNotFound => {
                (StatusCode::NOT_FOUND, JsonMsg::new("User not found")).into_response()
            }
            Self::UsernameTaken => {
                (StatusCode::CONFLICT, JsonMsg::new("Username already taken")).into_response()
            }
            Self::EmailTaken => {
                (StatusCode::CONFLICT, JsonMsg::new("Email already taken")).into_response() 
            }
            Self::WrongPassword => {
                (StatusCode::UNAUTHORIZED, JsonMsg::new("Password didn't match")).into_response()
            }
            Self::SessionExpired => {
                (StatusCode::UNAUTHORIZED, JsonMsg::new("Your session has expired")).into_response()
            }
            Self::InvalidSession(set_cookies) => {
                (StatusCode::UNAUTHORIZED, set_cookies, JsonMsg::new("Invalid Session")).into_response()
            }
            Self::RefreshSession(set_cookies) => {
                (StatusCode::OK, set_cookies, JsonMsg::new("Session Refreshed")).into_response()
            }
            Self::ServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, JsonMsg::new("Something went wrong")).into_response()
            }
        }
    }
}

#[derive(serde::Serialize)]
pub struct JsonMsg<'a> {
    message: &'a str,
}

impl<'a> JsonMsg<'a> {
    #[inline]
    pub fn new(error: &'a str) -> axum::Json<Self> {
        axum::Json(Self { message: error })
    }
}
