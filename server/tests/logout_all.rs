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

    let endpoint = format!("{}/api/user/logout_all", SOCKET);
    out.write("Enter cookie: ");
    let cookies = token.next_line::<String>();
    let res = client
        .post(&endpoint)
        .header(header::COOKIE, cookies)
        .send()?;
    writeln!(out.inner, "{:?}", res.text()?);

    Ok(())
}
