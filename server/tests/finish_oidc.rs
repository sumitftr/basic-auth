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

    let endpoint4 = format!("{}/api/register/finish_oidc", SOCKET);
    loop {
        out.write("Enter your email: ");
        let email = token.next::<String>();
        // out.write("Enter year of birth: ");
        // let year = token.next::<u32>();
        // out.write("Enter month of birth: ");
        // let month = token.next::<u8>();
        // out.write("Enter day of birth: ");
        // let day = token.next::<u8>();
        out.write("Enter your username: ");
        let username = token.next::<String>();
        let body4 = format!(r#"{{"email":"{email}","username":"{username}"}}"#);
        let res4 = client
            .post(&endpoint4)
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
                        .get(reqwest::header::SET_COOKIE)
                        .unwrap()
                        .to_str()
                        .map(|v| v[..v.find(';').unwrap()].to_string())
                        .unwrap();
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
