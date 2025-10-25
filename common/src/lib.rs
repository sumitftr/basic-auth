mod error;
pub use error::AppError;

pub mod mail;
pub mod user_session;
pub mod validation;

pub static SERVICE_NAME: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("DATABASE_URI").unwrap());

pub static SECRET_KEY: std::sync::LazyLock<Vec<u8>> =
    std::sync::LazyLock::new(|| Vec::from(std::env::var("SECRET_KEY").unwrap()));

// pub fn create_session(user_agent: String) -> (UserSession, ActiveUserSession, HeaderMap) {
//     let mut jar = CookieJar::new();
//     // session id cookie
//     let ssid = Cookie::build(("SSID", uuid::Uuid::new_v4().to_string()))
//         .expires(cookie::Expiration::DateTime(OffsetDateTime::now_utc()))
//         .http_only(true)
//         .same_site(cookie::SameSite::None)
//         .secure(true)
//         .path("/")
//         .domain("crates.io")
//         .max_age(cookie::time::Duration::days(30))
//         .build();

//     // creating cookie jar without signing cookie
//     jar.add(ssid.clone());
//     let uuids = jar
//         .delta()
//         .map(|c| {
//             let s = c.to_string();
//             s[0..s.find(';').unwrap()].to_string()
//         })
//         .collect::<Vec<String>>();

//     // creating final cookie jar
//     jar.signed_mut(&crate::SECRET_KEY).add(ssid);
//     let set_cookie_values = jar.delta().map(|c| c.to_string()).collect::<Vec<String>>();

//     (
//         UserSession {
//             uuids,
//             expires: SystemTime::now() + Duration::from_secs(30 * 86400),
//             user_agent,
//         },
//         ActiveUserSession {
//             cookies: set_cookie_values
//                 .iter()
//                 .map(|c| c[0..c.find(';').unwrap()].to_string())
//                 .collect(),
//         },
//         set_cookie_values
//             .into_iter()
//             .map(|c| (header::SET_COOKIE, HeaderValue::from_str(&c).unwrap()))
//             .collect::<HeaderMap>(),
//     )
// }
