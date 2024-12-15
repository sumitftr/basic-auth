pub mod database;
pub mod models;
pub mod routes;
pub mod utils;

use std::sync::LazyLock;

pub static DATABASE_URI: LazyLock<String> =
    LazyLock::new(|| std::env::var("DATABASE_URI").unwrap());

pub static SECRET_KEY: LazyLock<String> =
    LazyLock::new(|| std::env::var("SUPER_SECRET_KEY").unwrap());
