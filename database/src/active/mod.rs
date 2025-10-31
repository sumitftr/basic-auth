use crate::user::User;
use common::user_session::ActiveUserSession;
use std::sync::Arc;

impl crate::Db {
    pub fn make_user_active(self: &Arc<Self>, active_user_session: ActiveUserSession, user: User) {
        self.active.insert(active_user_session, user);
    }

    pub fn get_active_user(
        self: &Arc<Self>,
        active_user_session: &ActiveUserSession,
    ) -> Option<User> {
        self.active.get(active_user_session)
    }

    pub fn remove_active_user(
        self: &Arc<Self>,
        active_user_session: &ActiveUserSession,
    ) -> Option<User> {
        self.active.remove(active_user_session)
    }
}
