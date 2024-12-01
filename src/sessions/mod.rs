#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

pub fn make_token(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: username.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("secret".as_bytes()),
    )
}

pub fn check_token(headers_map: &axum::http::HeaderMap) -> bool {
    if let Some(auth_header) = headers_map.get("Authorization") {
        if let Ok(token) = auth_header.to_str() {
            // if auth_header_str.starts_with("Bearer ") {
            //     let token = auth_header_str.trim_start_matches("Bearer ").to_string();
            match jsonwebtoken::decode::<Claims>(
                &token,
                &jsonwebtoken::DecodingKey::from_secret("secret".as_ref()),
                &jsonwebtoken::Validation::default(),
            ) {
                Ok(_) => return true,
                Err(_) => return false,
            }
            // }
        }
    }

    false
}
