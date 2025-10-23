mod error;
pub use error::AppError;

pub mod mail;
pub mod user_session;
pub mod validation;

pub static SERVICE_NAME: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("DATABASE_URI").unwrap());
