#![allow(unused_must_use)]
mod common;

use common::{Printer, Scanner};
use fake::Fake;
use reqwest::header;
use std::io::Write;

#[test]
fn main() -> Result<(), reqwest::Error> {
    const SOCKET: &str = "http://127.0.0.1:8080";
    let client = reqwest::blocking::Client::builder()
        .user_agent(fake::faker::internet::en::UserAgent().fake::<String>())
        .build()
        .unwrap_or_default();

    // for io
    let mut token = Scanner::new(std::io::stdin().lock());
    let mut out = Printer::new();

    let mut email: String;
    let endpoint1 = format!("{}/api/forgot_password", SOCKET);

    loop {
        out.write("Enter email: ");
        email = token.next_line::<String>();
        let body1 = format!(r#"{{"email":"{email}"}}"#);
        let res1 = client
            .post(&endpoint1)
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

    loop {
        out.write("Enter your code: ");
        let code = token.next::<String>();
        out.write("Enter your password: ");
        let password = token.next_line::<String>();
        let body2 = format!(r#"{{"password": "{password}"}}"#);
        let endpoint2 = format!(r#"{}/api/reset_password?code={}"#, SOCKET, code);
        let res2 = client
            .post(&endpoint2)
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

    Ok(())
}
