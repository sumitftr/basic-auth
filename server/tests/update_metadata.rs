#![allow(unused_must_use)]
mod common;

use common::{Printer, Scanner};
use fake::{
    Fake,
    faker::{
        address::en::CountryName,
        name::en::{FirstName, LastName},
    },
};
use reqwest::header;
use std::io::Write;

#[test]
fn main() -> Result<(), reqwest::Error> {
    const SOCKET: &str = "http://127.0.0.1:8080";
    let is_auto = std::env::var("AUTO").ok().map(|v| v.as_str() == "true").unwrap_or(false);
    let client = reqwest::blocking::Client::builder()
        .user_agent(fake::faker::internet::en::UserAgent().fake::<String>())
        .build()
        .unwrap_or_default();

    let genders: Vec<String> = vec![
        String::from("Male"),
        String::from("Female"),
        String::from("Non-binary"),
        String::from("Genderqueer"),
        String::from("Genderfluid"),
        String::from("Agender"),
        String::from("Bigender"),
        String::from("Two-Spirit"),
        String::from("Transgender"),
        String::from("Cisgender"),
        String::from("Pangender"),
        String::from("Demigender"),
        String::from("Neutrois"),
        String::from("Androgyne"),
        String::from("Third Gender"),
        String::from("Prefer not to say"),
        String::from("Custom"),
    ];

    // for io
    let mut token = Scanner::new(std::io::stdin().lock());
    let mut out = Printer::new();

    out.write("Enter (L, B, G, C) for updating (legal name, birth date, gender, country): ");
    let opt = token.next_line::<String>();

    out.write("Enter cookie: ");
    let cookies = token.next_line::<String>();

    loop {
        let (endpoint, body) = match opt.as_str() {
            "L" => {
                let endpoint = format!("{}/api/settings/legal_name", SOCKET);
                let legal_name = if is_auto {
                    format!("{} {}", FirstName().fake::<String>(), LastName().fake::<String>())
                } else {
                    out.write("Enter legal name: ");
                    token.next_line::<String>()
                };
                let body = format!(r#"{{"legal_name": "{legal_name}"}}"#);
                (endpoint, body)
            }
            "B" => {
                let endpoint = format!("{}/api/settings/birth_date", SOCKET);
                let (year, month, day, offset_hours, offset_minutes, offset_seconds) = if is_auto {
                    (
                        (1950..=2024).fake::<u32>(),
                        (1..=12).fake::<u8>(),
                        (1..=28).fake::<u8>(),
                        (1..=11).fake::<u8>(),
                        (1..=59).fake::<u8>(),
                        (1..=59).fake::<u8>(),
                    )
                } else {
                    out.write("Enter year: ");
                    let year = token.next_line::<u32>();
                    out.write("Enter month: ");
                    let month = token.next_line::<u8>();
                    out.write("Enter day: ");
                    let day = token.next_line::<u8>();
                    let offset_hours = (1..=11).fake::<u8>();
                    let offset_minutes = (1..=59).fake::<u8>();
                    let offset_seconds = (1..=59).fake::<u8>();
                    (year, month, day, offset_hours, offset_minutes, offset_seconds)
                };
                let body = format!(
                    r#"{{"year": {year}, "month": {month}, "day": {day}, "offset_hours": {offset_hours}, "offset_minutes": {offset_minutes}, "offset_seconds": {offset_seconds}}}"#
                );
                (endpoint, body)
            }
            "G" => {
                let endpoint = format!("{}/api/settings/gender", SOCKET);
                let gender = if is_auto {
                    genders[(0..genders.len()).fake::<usize>()].clone()
                } else {
                    out.write("Enter gender: ");
                    token.next_line::<String>()
                };
                let body = format!(r#"{{"gender": "{gender}"}}"#);
                (endpoint, body)
            }
            "C" => {
                let endpoint = format!("{}/api/settings/country", SOCKET);
                let country = if is_auto {
                    CountryName().fake::<String>()
                } else {
                    out.write("Enter country: ");
                    token.next_line::<String>()
                };
                let body = format!(r#"{{"country": "{country}"}}"#);
                (endpoint, body)
            }
            _ => {
                panic!("What the fuck bro...?? please enter a given option.")
            }
        };

        if is_auto {
            out.write(&body);
            out.write("\n");
        }
        let res1 = client
            .post(&endpoint)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, &cookies)
            .body(body)
            .send();
        match res1 {
            Ok(v) => {
                if v.status().is_client_error() {
                    writeln!(out.inner, "{:?}", v.text()?);
                } else {
                    break;
                }
            }
            Err(e) => {
                writeln!(out.inner, "{e:?}");
            }
        }
    }

    Ok(())
}
