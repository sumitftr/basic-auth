use std::time::SystemTime;

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

impl std::ops::Drop for User {
    fn drop(&mut self) {
        let mut synced_sessions = Vec::with_capacity(self.sessions.len());
        let now = SystemTime::now();
        for session in self.sessions.iter() {
            if now < session.expires {
                synced_sessions.push(session.to_owned());
            }
        }
        self.sessions = synced_sessions;
        // tokio::spawn(async move {
        //     crate::Db::new()
        //         .await
        //         .update_sessions(&self.username, &self.sessions)
        //         .await;
        // });
    }
}
