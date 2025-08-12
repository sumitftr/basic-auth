use std::sync::Arc;

// implementation block for deleted sessions
impl super::Db {
    pub fn is_session_revoked(self: Arc<Self>, token: &str) -> bool {
        let guard = self.banned_tokens.lock().unwrap();

        match guard.get(token) {
            Some(_) => false,
            None => true,
        }
    }

    pub async fn revoke_session(self: Arc<Self>, token: &str) {
        let mut guard = self.banned_tokens.lock().unwrap();

        guard.insert(token.to_string());
    }

    pub async fn remove_revoked_session(self: Arc<Self>, token: &str) {
        let mut guard = self.banned_tokens.lock().unwrap();

        guard.remove(token);
    }
}
