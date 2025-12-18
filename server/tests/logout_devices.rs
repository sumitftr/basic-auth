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

    out.write("Enter cookie: ");
    let cookies = token.next_line::<String>();

    let s_endpoint = format!("{}/api/settings", SOCKET);
    let res = client.get(&s_endpoint).header(header::COOKIE, &cookies).send()?;
    writeln!(out.inner, "{:#?}", res.text()?);

    let endpoint = format!("{}/api/logout_devices", SOCKET);
    out.write("Enter number of cookies you want to delete: ");
    let l = token.next_line::<usize>();
    let mut sessions = Vec::with_capacity(l);
    for _ in 0..l {
        sessions.push(token.next::<String>());
    }
    let res = client
        .post(&endpoint)
        .header(header::COOKIE, &cookies)
        .json(&LogoutDevicesRequest { sessions })
        .send()?;
    writeln!(out.inner, "{:?}", res.text()?);

    Ok(())
}

#[derive(serde::Serialize)]
pub struct LogoutDevicesRequest {
    sessions: Vec<String>,
}
