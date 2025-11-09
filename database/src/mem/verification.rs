use std::sync::Arc;

// implementation block for users, those who are trying to update email or phone
impl crate::Db {
    #[inline]
    pub fn add_verification_entry(self: &Arc<Self>, old: String, new: String, otp: String) {
        self.verification.insert(old, (new, otp));
    }

    #[inline]
    pub fn get_verification_entry(self: &Arc<Self>, old: &str) -> Option<(String, String)> {
        self.verification.get(old)
    }

    #[inline]
    pub fn remove_verification_entry(self: &Arc<Self>, old: &str) {
        self.verification.remove(old);
    }
}
