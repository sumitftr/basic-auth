mod from_oidc;
mod registration;
mod update;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Applicant {
    pub socket_addr: std::net::SocketAddr,
    pub display_name: Option<String>,
    pub email: String,
    pub birth_date: Option<mongodb::bson::DateTime>,
    pub password: Option<String>,
    pub icon: Option<String>,
    pub phone: Option<String>,
    pub oauth_provider: Option<common::oauth::OAuthProvider>,
    pub status: ApplicationStatus,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "tag", content = "value")]
pub enum ApplicationStatus {
    Created(String), // OTP
    EmailVerified,
    PasswordSet,
    OidcVerified,
    UpdatingEmail { old_email: String, otp: String }, // OTP
    UpdatingPhone { old_phone: String, otp: String }, // OTP
}

impl crate::Db {
    pub async fn remove_applicant(self: std::sync::Arc<Self>, socket_addr: std::net::SocketAddr) {
        use mongodb::bson;
        use std::net::SocketAddr;
        let filter = match socket_addr {
            SocketAddr::V4(v4_addr) => {
                let octets = v4_addr.ip().octets();
                bson::doc! {
                    "socket_addr": { "$elemMatch": { "V4": [
                        [octets[0] as i32, octets[1] as i32, octets[2] as i32, octets[3] as i32],
                        v4_addr.port() as i32
                    ] } }
                }
            }
            SocketAddr::V6(v6_addr) => {
                let segments = v6_addr.ip().segments();
                bson::doc! {
                    "socket_addr": { "$elemMatch": { "V6": [
                        segments.iter().map(|&s| s as i32).collect::<Vec<_>>(),
                        v6_addr.port() as i32,
                        v6_addr.flowinfo() as i32,
                        v6_addr.scope_id() as i32
                    ] } }
                }
            }
        };
        dbg!(&filter);
        match self.applicants.delete_one(filter).await {
            Ok(v) => tracing::info!("{v:?}"),
            Err(e) => tracing::info!("{e:?}"),
        };
    }
}
