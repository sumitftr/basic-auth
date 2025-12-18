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

    out.write("Enter username: ");
    let username = token.next_line::<String>();
    out.write("Enter password: ");
    let password = token.next_line::<String>();
    let body = format!(r#"{{"username": "{username}", "password": "{password}"}}"#);
    let res = client
        .post(format!("{}/api/login", SOCKET))
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()?;
    let cookies = v
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .into_iter()
        .map(|s| {
            let v = s.to_str().unwrap();
            v[..v.find(';').unwrap()].to_string()
        })
        .collect::<Vec<String>>()
        .join(";");
    writeln!(out.inner, "{cookies}");
    writeln!(out.inner, "{:?}", res.text()?);

    Ok(())
}
