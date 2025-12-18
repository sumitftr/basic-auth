#![allow(unused_must_use)]
mod common;

use common::{Printer, Scanner};
use fake::{Fake, faker::internet::en::FreeEmail};
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
    let mut new_email: String;

    out.write("Enter cookie: ");
    let cookies = token.next_line::<String>();

    loop {
        if is_auto {
            new_email = FreeEmail().fake::<String>()
        } else {
            out.write("Enter email: ");
            new_email = token.next_line::<String>();
        };

        let body1 = format!(r#"{{"new_email": "{new_email}"}}"#);
        if is_auto {
            out.write(&body1);
            out.write("\n");
        }
        let res1 = client
            .post(format!("{}/api/settings/email", SOCKET))
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, &cookies)
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

    loop {
        out.write("Enter otp: ");
        let otp = token.next_line::<String>();
        let body2 = format!(r#"{{"new_email": "{new_email}", "otp": "{otp}"}}"#);
        let res2 = client
            .post(format!("{}/api/settings/verify_email", SOCKET))
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, &cookies)
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

    Ok(())
}
