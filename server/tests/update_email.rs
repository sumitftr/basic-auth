#![allow(unused_must_use)]
mod common;

use common::{Printer, Scanner};
use reqwest::header;
use std::io::Write;

#[test]
fn main() -> Result<(), reqwest::Error> {
    const SOCKET: &str = "http://127.0.0.1:8080";
    let client = reqwest::blocking::Client::builder()
        .user_agent("reqwest 0.12, rust lang")
        .build()
        .unwrap_or_default();

    // for io
    let mut token = Scanner::new(std::io::stdin().lock());
    let mut out = Printer::new();

    out.write("Enter cookie: ");
    let cookies = token.next_line::<String>();

    loop {
        let endpoint1 = format!("{}/api/settings/email", SOCKET);
        out.write("Enter email: ");
        let new_email = token.next_line::<String>();
        let body1 = format!(r#"{{"new_email":"{new_email}"}}"#);
        let res1 = client
            .post(&endpoint1)
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
        let endpoint2 = format!("{}/api/settings/verify_email", SOCKET);
        out.write("Enter otp: ");
        let otp = token.next_line::<String>();
        let body2 = format!(r#"{{"otp":"{otp}"}}"#);
        let res2 = client
            .post(&endpoint2)
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
