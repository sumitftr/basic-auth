use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

pub mod user;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub _id: ObjectId,
    pub name: String,
    pub email: String,
    pub dob: DateTime,
    pub password: String,
    pub username: String,
    pub gender: Option<String>,
    pub phone: Option<String>,
    // status: UserStatus,
    pub created: DateTime,
    pub last_login: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserStatus {
    Public,
    Private,
    Blocked,
}
