use crate::database::DBConf;
use axum::{
    extract::{ConnectInfo, State},
    http::{header, Request, StatusCode},
    response::Response,
};
use jsonwebtoken::errors::ErrorKind;
use mongodb::bson::DateTime;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub iat: String, // issued at
    pub exp: String, // expiration time
    pub ip: String,  // client ip
}

// For more information on claims visit:
// https://www.iana.org/assignments/jwt/jwt.xhtml

pub fn generate(username: &str, ip: String) -> Result<String, String> {
    let cur_time = SystemTime::now();
    let claims = Claims {
        sub: username.to_string(),
        iat: DateTime::from_system_time(cur_time)
            .try_to_rfc3339_string()
            .unwrap(),
        exp: DateTime::from_system_time(cur_time + Duration::new(3600, 0)).to_string(),
        ip,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(&*crate::SECRET_KEY.as_bytes()),
    )
    .map_err(|e| {
        let result = format!("Failed to create token");
        tracing::error!(result, username, "{e:?}");
        result
    })
}

// the middleware that validates requested user tokens
pub async fn authenticate(
    State(state): State<Arc<DBConf>>,
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    mut req: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<Response, (StatusCode, String)> {
    // checking for the authorization header
    let Some(auth_header) = req.headers().get(header::AUTHORIZATION) else {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Authorization header not set"),
        ));
    };

    // stripping bearer from authorization header
    let token = auth_header
        .to_str()
        .map(|v| v.strip_prefix("Bearer ").unwrap())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, format!("")))?;

    // checking if the token is banned
    if Arc::clone(&state).is_token_banned(token).await {
        return Err((StatusCode::BAD_REQUEST, "Token has expired".to_string()));
    }

    // decoding jwt token
    let tokendata = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(&*crate::SECRET_KEY.as_ref()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|e| match e.into_kind() {
        ErrorKind::InvalidToken => (StatusCode::BAD_REQUEST, format!("Token Invalid")),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Authentication Failed"),
        ),
    })?;

    // checking expiration time
    let exp_time = DateTime::parse_rfc3339_str(tokendata.claims.exp)
        .unwrap()
        .to_system_time();
    if SystemTime::now() > exp_time {
        // removing if the token is present in `banned_tokens`
        state.remove_banned_token(token).await;
        return Err((StatusCode::UNAUTHORIZED, "Token has expired".to_string()));
    }

    // checking ip
    if tokendata.claims.ip != conn_info.ip() {
        return Err((StatusCode::BAD_REQUEST, "Token Invalid".to_string()));
    }

    todo!()
}
