use crate::AppError;
use std::str::FromStr;
use time::OffsetDateTime;

#[allow(unused)]
const SPECIAL: [char; 33] = [
    ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<',
    '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
];

pub fn is_birth_date_valid(
    year: u32,
    month: u8,
    day: u8,
    offset: time::UtcOffset,
) -> Result<OffsetDateTime, AppError> {
    // Convert month number to time::Month enum
    let month_enum = match time::Month::try_from(month) {
        Ok(m) => m,
        Err(_) => {
            tracing::error!("Invalid month: {month}");
            return Err(AppError::InvalidData("Invalid month"));
        }
    };

    // Try to create a valid date
    let date = match time::Date::from_calendar_date(year as i32, month_enum, day) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Invalid date: year={year}, month={month}, day={day}, error={e:?}");
            return Err(AppError::InvalidData("Invalid Birth Date"));
        }
    };

    // Create a OffsetDateTime at midnight
    let birth_datetime = OffsetDateTime::new_in_offset(date, time::Time::MIDNIGHT, offset);

    // Check if birth date is in the future
    if birth_datetime > OffsetDateTime::now_utc() {
        tracing::error!("Birth date is in the future: {:?}", birth_datetime);
        return Err(AppError::InvalidData("Date of Birth cannot be in the future"));
    }

    Ok(birth_datetime)
}

pub fn is_password_strong(p: &str) -> Result<(), AppError> {
    if p.len() < 8 {
        return Err(AppError::InvalidData("Password cannot be less than 8 characters"));
    }
    if p.len() > 128 {
        return Err(AppError::InvalidData("Password cannot be more than 128 characters"));
    }
    let (mut lower, mut upper, mut digit) = (false, false, false);
    for c in p.chars() {
        if c.is_lowercase() {
            lower = true;
        }
        if c.is_uppercase() {
            upper = true;
        }
        if c.is_numeric() {
            digit = true;
        }
    }
    if lower && upper && digit {
        return Ok(());
    }
    Err(AppError::InvalidData(
        "Password must contain a lowercase alphabet, a uppercase alphabet and a digit",
    ))
}

// a valid name contains two or more words
// each words should only contain english alphabets
pub fn is_legal_name_valid(s: &str) -> Result<String, AppError> {
    if s.len() > 256 {
        return Err(AppError::InvalidData("Legal Name should be lesser than 256 characters"));
    }
    let mut result = String::new();
    let mut count = 0;
    for part in s.split_whitespace() {
        if part.chars().any(|b| !b.is_alphabetic()) {
            return Err(AppError::InvalidData("Only alphabets are allowed inside name"));
        }
        if !result.is_empty() {
            result.push(' ');
        }
        count += 1;
        result.push_str(part);
    }
    if !result.is_empty() && count >= 2 {
        Ok(result)
    } else {
        Err(AppError::InvalidData("Name must contain two or more words"))
    }
}

pub fn is_display_name_valid(display_name: &str) -> Result<(), AppError> {
    if display_name.trim().is_empty() {
        return Err(AppError::InvalidData("Name cannot be empty"));
    }
    if display_name.len() > 64 {
        return Err(AppError::InvalidData("Name should be lesser than 64 characters"));
    }
    if !display_name.trim().is_ascii() {
        return Err(AppError::InvalidData("Name can only contain ascii characters"));
    }
    Ok(())
}

pub fn is_bio_valid(bio: &str) -> Result<(), AppError> {
    if bio.len() > 3000 {
        return Err(AppError::InvalidData("Bio too long (MAX: 3000)"));
    }
    Ok(())
}

pub fn is_gender_valid(gender: &str) -> Result<(), AppError> {
    if gender.chars().any(|c| !c.is_ascii_alphabetic()) {
        return Err(AppError::InvalidData("Invalid Gender"));
    }
    Ok(())
}

pub fn is_country_valid(country: &str) -> Result<String, AppError> {
    let c = celes::Country::from_str(country.trim()).map_err(|e| {
        tracing::error!("{e:?}");
        AppError::InvalidData("Invalid Country")
    })?;
    Ok(c.long_name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! name_test {
        ($($name:ident: $exp:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (haystack, expected) = $exp;
                    assert_eq!(is_legal_name_valid(haystack).ok(), expected);
                }
            )*
        };
    }

    name_test! {
        name_test1: ("hello ", None),
        name_test2: ("   hello ", None),
        name_test3: ("   AB", None),
        name_test4: ("A", None),
        name_test5: ("SUMIT BRUH", Some("SUMIT BRUH".to_string())),
        name_test6: ("Sumit |", None),
        name_test7: (" RUST LANG", Some("RUST LANG".to_string())),
        name_test8: ("  hello  world broo  ", Some("hello world broo".to_string())),
        name_test9: ("RUST LANG   ", Some("RUST LANG".to_string())),
        name_test10: (" abc 82  cd  ", None),
        name_test11: (" abc_def  ", None),
        name_test12: (" abc-def  ", None),
        name_test13: (" abc@def  ", None),
    }
}
