use crate::users::User;
use common::session::{ParsedSession, Session};
use std::sync::{Arc, Mutex};

// implementation block for creating active users
// those are the users whose session is cached in memory
impl crate::Db {
    pub fn make_user_active(
        self: &Arc<Self>,
        user: User,
        session: Session,
    ) -> Arc<Mutex<(User, Vec<Session>)>> {
        let uid = user.id;
        let arc_wrapped = Arc::new(Mutex::new((user, vec![session])));
        self.active.insert(uid, arc_wrapped.clone());
        arc_wrapped
    }

    /// returns None if the user is not present
    /// returns Some(_, false) if the user is present but the session isn't
    /// returns Some(_, true) if the user and session is present
    pub fn get_active_user(
        self: &Arc<Self>,
        parsed_session: &ParsedSession,
    ) -> Option<(Arc<Mutex<(User, Vec<Session>)>>, bool)> {
        let arc_wrapped = self.active.get(&parsed_session.user_id)?;
        let mut flag = false;
        let guard = arc_wrapped.lock().unwrap();
        for i in guard.1.iter() {
            if i.unsigned_ssid == parsed_session.unsigned_ssid {
                flag = true;
            }
        }
        drop(guard);
        Some((arc_wrapped, flag))
    }

    pub fn remove_active_user(
        self: &Arc<Self>,
        parsed_session: &ParsedSession,
    ) -> Option<Arc<Mutex<(User, Vec<Session>)>>> {
        let arc_wrapped = self.active.get(&parsed_session.user_id)?;
        let mut guard = arc_wrapped.lock().unwrap();
        guard.1 =
            guard.1.drain(..).filter(|v| v.unsigned_ssid != parsed_session.unsigned_ssid).collect();
        if guard.1.is_empty() {
            self.active.remove(&parsed_session.user_id)
        } else {
            drop(guard);
            Some(arc_wrapped)
        }
    }
}
