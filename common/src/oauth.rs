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
    None,
}

impl From<&str> for OAuthProvider {
    fn from(provider: &str) -> Self {
        match provider {
            google if google == (OAuthProvider::Google).get_str() => OAuthProvider::Google,
            _ => OAuthProvider::None,
        }
    }
}

impl From<String> for OAuthProvider {
    fn from(value: String) -> Self {
        OAuthProvider::from(value.as_str())
    }
}

impl OAuthProvider {
    pub fn get_scopes(&self) -> Option<&'static str> {
        match self {
            OAuthProvider::Google => Some("openid email profile"),
            OAuthProvider::None => None,
        }
    }

    pub fn get_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::None => "",
        }
    }
}

pub fn get_oauth_provider(provider: OAuthProvider) -> Option<Arc<OAuthConfig<'static>>> {
    match provider {
        OAuthProvider::Google => Some(OAUTH_PROVIDERS.google.clone()),
        OAuthProvider::None => None,
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
        <&str as sqlx::Encode<'_, sqlx::Postgres>>::encode_by_ref(&self.get_str(), buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for OAuthProvider {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(OAuthProvider::from(s.as_str()))
    }
}
