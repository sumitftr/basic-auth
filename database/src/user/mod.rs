use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};

mod create;
mod delete;
mod read;
mod unregistered;
mod update;

#[derive(Serialize, Deserialize, Debug)]
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
    pub sessions: Vec<Vec<String>>,
    // status: UserStatus,
    // pub created: DateTime,
    // pub last_login: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserStatus {
    Normal,
    Locked,
    Blocked,
    Deactivated,
}

#[derive(Deserialize, Debug)]
pub struct UnregisteredEntry {
    pub name: String,
    // pub email: String,
    pub dob: DateTime,
    pub otp: u32,
    pub password: Option<String>,
    pub register_status: RegisterStatus,
    pub session: Vec<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub enum RegisterStatus {
    Created,
    EmailVerified,
    PasswordSet,
}
