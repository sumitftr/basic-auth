use crate::users::User;
use common::session::ActiveSession;
use std::sync::{Arc, Mutex};

// implementation block for creating active users
// those are the users whose session is cached in memory
impl crate::Db {
    #[inline]
    pub fn make_user_active(
        self: &Arc<Self>,
        active_session: ActiveSession,
        user: User,
    ) -> Arc<Mutex<User>> {
        let arc_wrapped_user = Arc::new(Mutex::new(user));
        self.active.insert(active_session, arc_wrapped_user.clone());
        arc_wrapped_user
    }

    #[inline]
    pub fn get_active_user(
        self: &Arc<Self>,
        active_session: &ActiveSession,
    ) -> Option<Arc<Mutex<User>>> {
        self.active.get(active_session)
    }

    #[inline]
    pub fn remove_active_user(
        self: &Arc<Self>,
        active_session: &ActiveSession,
    ) -> Option<Arc<Mutex<User>>> {
        self.active.remove(active_session)
    }
}
