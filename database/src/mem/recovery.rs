use std::sync::Arc;

// implementation block for those users who forgot their password
impl crate::Db {
    #[inline]
    pub fn add_recovery_entry(self: &Arc<Self>, code: String, email: String) {
        if let Some(v) = self.recovery.get(&email) {
            self.recovery.remove(&v);
        }
        self.recovery.insert(code.clone(), email.clone());
        self.recovery.insert(email, code);
    }

    #[inline]
    pub fn get_recovery_entry(self: &Arc<Self>, key: &str) -> Option<String> {
        self.recovery.get(key)
    }

    #[inline]
    pub fn remove_recovery_entry(self: &Arc<Self>, key: &str) {
        if let Some(another_key) = self.recovery.remove(key) {
            self.recovery.remove(&another_key);
        }
    }
}
