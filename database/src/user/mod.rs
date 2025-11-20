use common::session::Session;
use mongodb::bson::{DateTime, oid::ObjectId};

mod create;
mod delete;
mod read;
mod update;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct User {
    pub _id: ObjectId,
    pub legal_name: String,
    pub email: String,
    pub birth_date: DateTime,
    pub password: String,
    pub username: String,
    pub icon: Option<String>,
    pub display_name: String,
    pub bio: Option<String>,
    pub gender: Option<String>,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub sessions: Vec<Session>,
    pub created: DateTime,
    // pub last_accessed: DateTime,
}
