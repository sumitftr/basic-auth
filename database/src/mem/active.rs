use crate::user::User;
use common::user_session::ActiveUserSession;
use std::sync::{Arc, Mutex};

// implementation block for creating active users
// those are the users whose session is cached in memory
impl crate::Db {
    #[inline]
    pub fn make_user_active(
        self: &Arc<Self>,
        active_user_session: ActiveUserSession,
        user: User,
    ) -> Arc<Mutex<User>> {
        let wrapped_user = Arc::new(Mutex::new(user));
        self.active
            .insert(active_user_session, wrapped_user.clone());
        wrapped_user
    }

    #[inline]
    pub fn get_active_user(
        self: &Arc<Self>,
        active_user_session: &ActiveUserSession,
    ) -> Option<Arc<Mutex<User>>> {
        self.active.get(active_user_session)
    }

    #[inline]
    pub fn remove_active_user(
        self: &Arc<Self>,
        active_user_session: &ActiveUserSession,
    ) -> Option<Arc<Mutex<User>>> {
        self.active.remove(active_user_session)
    }
}
