use common::user_session::UserSession;
use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};

mod create;
mod delete;
mod read;
mod update;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub _id: ObjectId,
    pub legal_name: String,
    pub email: String,
    pub dob: DateTime,
    pub password: String,
    pub username: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub gender: Option<String>,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub sessions: Vec<UserSession>,
    pub created: DateTime,
    // status: UserStatus,
    // pub last_login: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserStatus {
    Normal,
    Locked,
    Blocked,
    Deactivated,
}
