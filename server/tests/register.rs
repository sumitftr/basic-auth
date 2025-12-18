#![allow(unused_must_use)]
mod common;

use common::{Printer, Scanner};
use fake::Fake;
use fake::faker::internet::en::{FreeEmail, Password, Username};
use fake::faker::name::en::{FirstName, LastName};
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

    // for io
    let mut token = Scanner::new(std::io::stdin().lock());
    let mut out = Printer::new();
    let mut email: String;

    // first step of registering
    loop {
        let (name, year, month, day, offset_hours, offset_minutes, offset_seconds) = if is_auto {
            email = FreeEmail().fake();
            (
                format!("{} {}", FirstName().fake::<String>(), LastName().fake::<String>()),
                (1950..=2024).fake::<u32>(),
                (1..=12).fake::<u8>(),
                (1..=28).fake::<u8>(),
                (1..=11).fake::<u8>(),
                (1..=59).fake::<u8>(),
                (1..=59).fake::<u8>(),
            )
        } else {
            out.write("Enter name: ");
            let name = token.next_line::<String>();
            out.write("Enter email: ");
            email = token.next_line::<String>();
            out.write("Enter year of birth: ");
            let year = token.next::<u32>();
            out.write("Enter month of birth: ");
            let month = token.next::<u8>();
            out.write("Enter day of birth: ");
            let day = token.next::<u8>();
            let offset_hours = (1..=11).fake::<u8>();
            let offset_minutes = (1..=59).fake::<u8>();
            let offset_seconds = (1..=59).fake::<u8>();
            (name, year, month, day, offset_hours, offset_minutes, offset_seconds)
        };

        let body1 = format!(
            r#"{{"name": "{name}", "email": "{email}", "year": {year}, "month": {month}, "day": {day}, "offset_hours": {offset_hours}, "offset_minutes": {offset_minutes}, "offset_seconds": {offset_seconds}}}"#
        );
        if is_auto {
            out.write(&body1);
            out.write("\n");
        }
        let res1 = client
            .post(format!("{}/api/register", SOCKET))
            .header(header::CONTENT_TYPE, "application/json")
            .body(body1)
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

    // second step of registering
    loop {
        out.write("Enter your otp: ");
        let otp = token.next::<String>();

        let body2 = format!(r#"{{"email": "{email}", "otp": "{otp}"}}"#);
        let res2 = client
            .post(format!("{}/api/register/verify_email", SOCKET))
            .header(header::CONTENT_TYPE, "application/json")
            .body(body2)
            .send();
        match res2 {
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

    // third step of registering
    loop {
        let password = if is_auto {
            Password(8..128).fake::<String>()
        } else {
            out.write("Enter your password: ");
            token.next_line::<String>()
        };
        let body3 = format!(r#"{{"email": "{email}", "password": "{password}"}}"#);
        if is_auto {
            out.write(&body3);
            out.write("\n");
        }

        let res3 = client
            .post(format!("{}/api/register/set_password", SOCKET))
            .header(header::CONTENT_TYPE, "application/json")
            .body(body3)
            .send();
        match res3 {
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

    // fourth step of registering
    loop {
        let username = if is_auto {
            Username().fake::<String>().replace("_", ".")
        } else {
            out.write("Enter your username: ");
            token.next::<String>()
        };

        let body4 = format!(r#"{{"email": "{email}", "username": "{username}"}}"#);
        if is_auto {
            out.write(&body4);
            out.write("\n");
        }
        let res4 = client
            .post(format!("{}/api/register/set_username", SOCKET))
            .header(header::CONTENT_TYPE, "application/json")
            .body(body4)
            .send();
        match res4 {
            Ok(v) => {
                if v.status().is_client_error() {
                    writeln!(out.inner, "{:?}", v.text()?);
                } else {
                    let cookies = v
                        .headers()
                        .get_all(reqwest::header::SET_COOKIE)
                        .into_iter()
                        .map(|s| {
                            let v = s.to_str().unwrap();
                            v[..v.find(';').unwrap()].to_string()
                        })
                        .collect::<Vec<String>>()
                        .join("; ");
                    writeln!(out.inner, "{cookies}");
                    writeln!(out.inner, "{:?}", v.text()?);
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
