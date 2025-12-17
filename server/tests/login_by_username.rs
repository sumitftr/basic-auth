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

    let endpoint = format!("{}/api/login", SOCKET);
    out.write("Enter username: ");
    let username = token.next_line::<String>();
    out.write("Enter password: ");
    let password = token.next_line::<String>();
    let body = format!(r#"{{"username": "{username}", "password": "{password}"}}"#);
    let res =
        client.post(&endpoint).header(header::CONTENT_TYPE, "application/json").body(body).send()?;
    let cookies = res
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .unwrap()
        .to_str()
        .map(|v| v[..v.find(';').unwrap()].to_string())
        .unwrap();
    writeln!(out.inner, "{cookies}");
    writeln!(out.inner, "{:?}", res.text()?);

    Ok(())
}
