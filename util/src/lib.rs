mod error;
pub mod generate;
pub mod mail;
pub mod oauth;
pub mod session;
pub mod validation;

pub use error::AppError;

pub static SERVICE_NAME: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("SERVICE_NAME").unwrap());

pub static SERVICE_DOMAIN: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("SERVICE_DOMAIN").unwrap());

pub static SECRET_KEY: std::sync::LazyLock<Vec<u8>> =
    std::sync::LazyLock::new(|| Vec::from(std::env::var("SECRET_KEY").unwrap()));
