use common::session::Session;
use mongodb::bson::{DateTime, oid::ObjectId};

mod create;
mod delete;
mod read;
mod update_by_email;
mod update_by_username;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct User {
    pub _id: ObjectId,
    pub display_name: String,
    pub email: String,
    pub birth_date: DateTime,
    pub password: Option<String>,
    pub username: String,
    pub banner: Option<String>,
    pub icon: Option<String>,
    pub bio: Option<String>,
    pub legal_name: Option<String>,
    pub gender: Option<String>,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub sessions: Vec<Session>,
    pub created: DateTime,
    // pub oauth: Option<OAuthDetails>,
    // pub last_accessed: DateTime,
}

pub struct OAuthDetails {
    pub access_token: String,
    pub refresh_token: String,
    pub provider: common::oauth::OAuthProvider,
}
