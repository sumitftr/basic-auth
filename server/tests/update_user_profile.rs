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
    out.write("Enter icon path: ");
    let icon_path = token.next_line::<String>();
    out.write("Enter display name: ");
    let display_name = token.next_line::<String>();
    out.write("Enter bio: ");
    let bio = token.next_line::<String>();

    // creating multipart form
    let mut form = reqwest::blocking::multipart::Form::new();

    if !icon_path.is_empty() {
        let icon = std::fs::read(&icon_path).unwrap();
        let icon_part = reqwest::blocking::multipart::Part::bytes(icon)
            .file_name(icon_path)
            .mime_str("application/octet-stream")
            .unwrap();
        form = form.part("icon", icon_part);
    }

    if !display_name.is_empty() {
        let display_name_part = reqwest::blocking::multipart::Part::text(display_name);
        form = form.part("display_name", display_name_part);
    }

    if !bio.is_empty() {
        let bio_part = reqwest::blocking::multipart::Part::text(bio);
        form = form.part("bio", bio_part);
    }

    let endpoint = format!("{}/api/user/profile", SOCKET);
    let res = client.post(&endpoint).header(header::COOKIE, cookies).multipart(form).send()?;
    writeln!(out.inner, "{:?}", res.text()?);

    Ok(())
}
