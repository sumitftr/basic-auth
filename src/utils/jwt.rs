use mongodb::bson::datetime::DateTime;
use std::time::{Duration, SystemTime};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    iss: String, // issuer
    iat: String, // issued at
    exp: String, // expiration time
    ip: String,
}

// For more information on claims visit:
// https://www.iana.org/assignments/jwt/jwt.xhtml

pub fn make_token(username: &str, ip: String) -> Result<String, jsonwebtoken::errors::Error> {
    let cur_time = SystemTime::now();
    let claims = Claims {
        iss: username.to_string(),
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
}

pub fn check_token(headers_map: &axum::http::HeaderMap) -> Result<bool, String> {
    if let Some(auth_header) = headers_map.get("Authorization") {
        if let Ok(token) = auth_header.to_str() {
            // if auth_header_str.starts_with("Bearer ") {
            //     let token = auth_header_str.trim_start_matches("Bearer ").to_string();
            match jsonwebtoken::decode::<Claims>(
                &token,
                &jsonwebtoken::DecodingKey::from_secret(&*crate::SECRET_KEY.as_ref()),
                &jsonwebtoken::Validation::default(),
            ) {
                Ok(tokendata) => {
                    // time expiration check
                    let exp_time = DateTime::parse_rfc3339_str(tokendata.claims.exp)
                        .unwrap()
                        .to_system_time();
                    if SystemTime::now() > exp_time {
                        return Ok(false);
                    }

                    // ip check

                    return Ok(true);
                }
                Err(e) => return Err(e.to_string()),
            }
            // }
        }
    }

    Err(format!("Authorization header not set"))
}
