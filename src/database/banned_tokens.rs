use std::sync::Arc;

// implementation block for deleted sessions
impl super::DBConf {
    pub fn is_token_banned(self: Arc<Self>, token: &str) -> bool {
        let guard = self.banned_tokens.lock().unwrap();

        match guard.get(token) {
            Some(_) => false,
            None => true,
        }
    }

    pub async fn ban_token(self: Arc<Self>, token: &str) {
        let mut guard = self.banned_tokens.lock().unwrap();

        guard.insert(token.to_string());
    }

    pub async fn remove_banned_token(self: Arc<Self>, token: &str) {
        let mut guard = self.banned_tokens.lock().unwrap();

        guard.remove(token);
    }
}
