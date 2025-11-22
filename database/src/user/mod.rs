use common::session::Session;
use mongodb::bson::{DateTime, oid::ObjectId};

mod create;
mod delete;
mod read;
mod update_by_email;
mod update_by_username;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct User {
    pub _id: ObjectId,
    pub legal_name: String,
    pub email: String,
    pub birth_date: Option<DateTime>,
    pub password: Option<String>,
    pub username: String,
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
}

pub enum OAuthDetails {
    Google {},
}
