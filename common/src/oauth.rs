use crate::AppError;
use reqwest::Url;
use std::sync::Arc;

static OAUTH_PROVIDERS: std::sync::LazyLock<OAuthProviders> =
    std::sync::LazyLock::new(OAuthProviders::default);

struct OAuthProviders<'a> {
    /// https://accounts.google.com/.well-known/openid-configuration
    pub google: Arc<OAuthConfig<'a>>,
}

pub struct OAuthConfig<'a> {
    pub client_id: String,
    pub client_secret: String,
    pub authorization_endpoint: Url,
    pub token_endpoint: &'a str,
    pub userinfo_endpoint: &'a str,
    pub provider: OAuthProvider,
}

impl<'a> Default for OAuthProviders<'a> {
    fn default() -> Self {
        Self {
            google: Arc::new(OAuthConfig {
                client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap(),
                client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap(),
                authorization_endpoint: Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
                    .unwrap(),
                token_endpoint: "https://oauth2.googleapis.com/token",
                userinfo_endpoint: "https://openidconnect.googleapis.com/v1/userinfo",
                provider: OAuthProvider::Google,
            }),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone, Debug)]
pub enum OAuthProvider {
    Google,
}

impl TryFrom<&str> for OAuthProvider {
    type Error = AppError;

    fn try_from(provider: &str) -> Result<Self, AppError> {
        match provider {
            google if google == (OAuthProvider::Google).as_str() => Ok(OAuthProvider::Google),
            _ => Err(AppError::BadReq("Invalid OAuth Provider")),
        }
    }
}

impl OAuthProvider {
    pub fn get_scopes(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "openid email profile",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
        }
    }
}

pub fn get_oauth_provider(provider: OAuthProvider) -> Arc<OAuthConfig<'static>> {
    match provider {
        OAuthProvider::Google => OAUTH_PROVIDERS.google.clone(),
    }
}

// Implement sqlx traits
impl sqlx::Type<sqlx::Postgres> for OAuthProvider {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for OAuthProvider {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<'_, sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for OAuthProvider {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        OAuthProvider::try_from(s.as_str())
            .map_err(|_| format!("Invalid OAuth provider: {}", s).into())
    }
}
