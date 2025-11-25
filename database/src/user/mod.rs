use common::session::Session;
use mongodb::bson::{DateTime, oid::ObjectId};

mod create;
mod delete;
mod read;
mod recovery;
mod registration;
mod update_by_email;
mod update_by_username;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct User {
    pub _id: ObjectId,
    pub legal_name: String,
    pub email: String,
    pub birth_date: Option<DateTime>,
    pub password: Option<String>,
    pub username: String,
    pub banner: Option<String>,
    pub icon: Option<String>,
    pub display_name: String,
    pub bio: Option<String>,
    pub gender: Option<String>,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub sessions: Vec<Session>,
    pub created: DateTime,
    // pub oauth: Option<OAuthDetails>,
    // pub last_accessed: DateTime,
    pub status: Option<UserStatus>,
}

pub struct OAuthDetails {
    pub access_token: String,
    pub refresh_token: String,
    pub provider: common::oauth::OAuthProvider,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "variant", content = "secret")]
pub enum UserStatus {
    Created(String), // OTP
    EmailVerified,
    PasswordSet,
    OidcVerified,
    Recovering(String),                           // HEX HASH
    UpdatingEmail { email: String, otp: String }, // OTP
    UpdatingPhone { phone: String, otp: String }, // OTP
}

impl Default for User {
    fn default() -> Self {
        Self {
            _id: ObjectId::new(),
            legal_name: "".to_string(),
            email: "".to_string(),
            birth_date: None,
            password: None,
            username: "".to_string(),
            banner: None,
            icon: None,
            display_name: "".to_string(),
            bio: None,
            gender: None,
            phone: None,
            country: None,
            sessions: Vec::new(),
            created: DateTime::now(),
            status: None,
        }
    }
}
