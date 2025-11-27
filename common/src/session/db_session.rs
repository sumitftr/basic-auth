use std::time::SystemTime;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Session {
    pub unsigned_ssid: String,
    pub expires: SystemTime,
    pub user_agent: String,
}

pub enum SessionStatus {
    Valid(u64),
    Expiring(u64),
    Refreshable(u64),
    Invalid,
}

impl Session {
    // timestamp in seconds
    pub const MEM_CACHE_DURATION: u64 = 28800; // 8 hours
    pub const MAX_REFRESH_DURATION: u64 = 604800; // 7 days

    /// returns the timestamp difference of the session with current time
    pub fn session_status(&self) -> SessionStatus {
        let diff = self.expires.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64
            - SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;

        if diff > 0 {
            if diff > Self::MEM_CACHE_DURATION as i64 {
                SessionStatus::Valid(diff as u64)
            } else {
                SessionStatus::Expiring(diff as u64)
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if -diff < Self::MAX_REFRESH_DURATION as i64 {
                SessionStatus::Refreshable(diff as u64)
            } else {
                SessionStatus::Invalid
            }
        }
    }
}
