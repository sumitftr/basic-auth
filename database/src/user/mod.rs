use mongodb::bson::{DateTime, oid::ObjectId};

mod create;
mod delete;
mod read;
mod update_by_email;
mod update_by_username;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
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
    pub oauth_provider: Option<common::oauth::OAuthProvider>,
    pub sessions: Vec<common::session::Session>,
    pub created: DateTime,
    // pub last_accessed: DateTime,
}
