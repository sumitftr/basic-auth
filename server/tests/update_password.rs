#![allow(unused_must_use)]
mod common;

use common::{Printer, Scanner};
use fake::{Fake, faker::internet::en::Password};
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

    out.write("Enter cookie: ");
    let cookies = token.next_line::<String>();

    loop {
        out.write("Enter old password: ");
        let old_password = token.next_line::<String>();
        let new_password = if is_auto {
            Password(8..128).fake::<String>()
        } else {
            out.write("Enter new password: ");
            token.next_line::<String>()
        };

        let body1 =
            format!(r#"{{"old_password": "{old_password}", "new_password": "{new_password}"}}"#);
        let res1 = client
            .post(format!("{}/api/settings/password", SOCKET))
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

    Ok(())
}
