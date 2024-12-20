use mongodb::bson::datetime::DateTime;
use std::time::{Duration, SystemTime};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    sub: String, // username
    iat: String, // issued at
    exp: String, // expiration time
    ip: String,  // client ip
}

// For more information on claims visit:
// https://www.iana.org/assignments/jwt/jwt.xhtml

pub fn make_token(username: &str, ip: String) -> Result<String, String> {
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

pub fn check_token(headers_map: &axum::http::HeaderMap, ip: String) -> Result<bool, String> {
    if let Some(auth_header) = headers_map.get("Authorization") {
        if let Ok(token) = auth_header.to_str() {
            let token = if token.starts_with("Bearer ") {
                token.trim_start_matches("Bearer ").to_string()
            } else {
                return Err("Bearer prefix not added".to_string());
            };

            // decoding token
            let tokendata = jsonwebtoken::decode::<Claims>(
                &token,
                &jsonwebtoken::DecodingKey::from_secret(&*crate::SECRET_KEY.as_ref()),
                &jsonwebtoken::Validation::default(),
            )
            .map_err(|e| e.to_string())?;

            // time expiration check
            let exp_time = DateTime::parse_rfc3339_str(tokendata.claims.exp)
                .unwrap()
                .to_system_time();
            if SystemTime::now() > exp_time {
                return Ok(false);
            }

            // ip check
            if tokendata.claims.ip != ip {
                return Ok(false);
            }

            return Ok(true);
        }
    }

    Err(format!("Authorization header not set"))
}
