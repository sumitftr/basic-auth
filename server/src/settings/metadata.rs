use axum::{Extension, Json, extract::State};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::{Db, UserData};
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct UpdateLegalNameRequest {
    legal_name: String,
}

pub async fn update_legal_name(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserData>,
    Json(body): Json<UpdateLegalNameRequest>,
) -> Result<ErasedJson, AppError> {
    common::validation::is_legal_name_valid(&body.legal_name)?;
    let username = user.lock().unwrap().0.username.clone();
    db.update_legal_name(&username, &body.legal_name).await?;
    user.lock().unwrap().0.legal_name = Some(body.legal_name.clone());
    Ok(json!({
        "legal_name": body.legal_name,
        "message": "Your legal name has been updated"
    }))
}

#[derive(serde::Deserialize)]
pub struct UpdateBirthDateRequest {
    year: u32,
    month: u8,
    day: u8,
    offset_hours: i8,
    offset_minutes: i8,
    offset_seconds: i8,
}

pub async fn update_birth_date(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserData>,
    Json(body): Json<UpdateBirthDateRequest>,
) -> Result<ErasedJson, AppError> {
    let offset =
        time::UtcOffset::from_hms(body.offset_hours, body.offset_minutes, body.offset_seconds)
            .map_err(|_| AppError::InvalidData("Invalid UTC Offset"))?;
    let birth_date =
        common::validation::is_birth_date_valid(body.year, body.month, body.day, offset)?;
    let username = user.lock().unwrap().0.username.clone();
    db.update_birth_date(&username, birth_date).await?;
    user.lock().unwrap().0.birth_date = Some(birth_date);
    Ok(json!({
        "birth_date": birth_date.to_string(),
        "message": "Your birth date has been updated"
    }))
}

#[derive(serde::Deserialize)]
pub struct UpdateGenderRequest {
    gender: String,
}

pub async fn update_gender(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserData>,
    Json(body): Json<UpdateGenderRequest>,
) -> Result<ErasedJson, AppError> {
    common::validation::is_gender_valid(&body.gender)?;
    let username = user.lock().unwrap().0.username.clone();
    db.update_gender(&username, &body.gender).await?;
    user.lock().unwrap().0.gender = Some(body.gender.clone());
    Ok(json!({
        "gender": body.gender,
        "message": "Your gender has been updated"
    }))
}

#[derive(serde::Deserialize)]
pub struct UpdateCountryRequest {
    country: String,
}

pub async fn update_country(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserData>,
    Json(body): Json<UpdateCountryRequest>,
) -> Result<ErasedJson, AppError> {
    let country = common::validation::is_country_valid(&body.country)?;
    let username = user.lock().unwrap().0.username.clone();
    db.update_country(&username, &country).await?;
    user.lock().unwrap().0.country = Some(country.clone());
    Ok(json!({
        "country": country,
        "message": "Your country has been updated"
    }))
}
