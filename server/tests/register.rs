#![allow(unused_must_use)]
mod common;

use common::{Printer, Scanner};
use reqwest::header;
use std::io::Write;

#[test]
fn main() -> Result<(), reqwest::Error> {
    const SOCKET: &str = "http://127.0.0.1:8080";
    let client = reqwest::blocking::Client::builder()
        .user_agent("reqwest 0.12, rust")
        .build()
        .unwrap_or_default();

    // for io
    let mut token = Scanner::new(std::io::stdin().lock());
    let mut out = Printer::new();

    let mut email: String;
    let endpoint1 = format!("{}/api/user/register/start", SOCKET);
    let endpoint2 = format!("{}/api/user/register/verify_email", SOCKET);
    let endpoint3 = format!("{}/api/user/register/set_password", SOCKET);
    let endpoint4 = format!("{}/api/user/register/set_username", SOCKET);

    loop {
        // first step of registering
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
        let body1 = format!(
            r#"{{"name":"{name}","email":"{email}","year":{year},"month":{month},"day":{day}}}"#
        );
        let res1 = client
            .post(&endpoint1)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body1)
            .send();
        match res1 {
            Ok(v) => {
                writeln!(out.inner, "{:?}", v.text()?);
                break;
            }
            Err(e) => {
                writeln!(out.inner, "{e:?}");
                continue;
            }
        }
    }

    loop {
        // second step of registering
        out.write("Enter your otp: ");
        let otp = token.next::<String>();
        let body2 = format!(r#"{{"email": "{email}", "otp": "{otp}"}}"#);
        let res2 = client
            .post(&endpoint2)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body2)
            .send();
        match res2 {
            Ok(v) => {
                writeln!(out.inner, "{:?}", v.text()?);
                break;
            }
            Err(e) => {
                writeln!(out.inner, "{e:?}");
                continue;
            }
        }
    }

    loop {
        // third step of registering
        out.write("Enter your password: ");
        let password = token.next_line::<String>();
        let body3 = format!(r#"{{"email": "{email}", "password": "{password}"}}"#);
        let res3 = client
            .post(&endpoint3)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body3)
            .send();
        match res3 {
            Ok(v) => {
                writeln!(out.inner, "{:?}", v.text()?);
                break;
            }
            Err(e) => {
                writeln!(out.inner, "{e:?}");
                continue;
            }
        }
    }

    loop {
        // fourth step of registering
        out.write("Enter your username: ");
        let username = token.next::<String>();
        let body4 = format!(r#"{{"email": "{email}", "username": "{username}"}}"#);
        let res4 = client
            .post(&endpoint4)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body4)
            .send();
        match res4 {
            Ok(v) => {
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
            Err(e) => {
                writeln!(out.inner, "{e:?}");
                continue;
            }
        }
    }

    Ok(())
}
