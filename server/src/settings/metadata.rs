use axum::{Extension, Json, extract::State};
use common::AppError;
use database::{Db, user::User};
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
pub struct UpdateBirthDateRequest {
    year: u32,
    month: u8,
    day: u8,
}

pub async fn update_birth_date(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<UpdateBirthDateRequest>,
) -> Result<String, AppError> {
    let birth_date = common::validation::is_birth_date_valid(body.year, body.month, body.day)?;
    let username = user.lock().unwrap().username.clone();
    db.update_birth_date(&username, birth_date).await?;
    user.lock().unwrap().birth_date = birth_date;
    Ok("Your birth date has been updated".to_string())
}

#[derive(serde::Deserialize)]
pub struct UpdateGenderRequest {
    gender: String,
}

pub async fn update_gender(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<UpdateGenderRequest>,
) -> Result<String, AppError> {
    common::validation::is_gender_valid(&body.gender)?;
    let username = user.lock().unwrap().username.clone();
    db.update_gender(&username, &body.gender).await?;
    user.lock().unwrap().gender = Some(body.gender);
    Ok("Your gender has been updated".to_string())
}

#[derive(serde::Deserialize)]
pub struct UpdateCountryRequest {
    country: String,
}

pub async fn update_country(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<UpdateCountryRequest>,
) -> Result<String, AppError> {
    let country = common::validation::is_country_valid(&body.country)?;
    let username = user.lock().unwrap().username.clone();
    db.update_country(&username, &country).await?;
    user.lock().unwrap().country = Some(country);
    Ok("Your country has been updated".to_string())
}
