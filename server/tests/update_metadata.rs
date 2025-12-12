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

    out.write("Enter (L, B, G, C) for updating (legal name, birth date, gender, country): ");
    let opt = token.next_line::<String>();

    out.write("Enter cookie: ");
    let cookies = token.next_line::<String>();

    loop {
        let (endpoint, body) = match opt.as_str() {
            "L" => {
                let endpoint = format!("{}/api/settings/legal_name", SOCKET);
                out.write("Enter legal name: ");
                let legal_name = token.next_line::<String>();
                let body = format!(r#"{{"legal_name":"{legal_name}"}}"#);
                (endpoint, body)
            }
            "B" => {
                let endpoint = format!("{}/api/settings/birth_date", SOCKET);
                out.write("Enter year: ");
                let year = token.next_line::<u32>();
                out.write("Enter month: ");
                let month = token.next_line::<u8>();
                out.write("Enter day: ");
                let day = token.next_line::<u8>();
                let body = format!(r#"{{"year":{year},"month":{month},"day":{day}}}"#);
                (endpoint, body)
            }
            "G" => {
                let endpoint = format!("{}/api/settings/gender", SOCKET);
                out.write("Enter gender: ");
                let gender = token.next_line::<String>();
                let body = format!(r#"{{"gender":"{gender}"}}"#);
                (endpoint, body)
            }
            "C" => {
                let endpoint = format!("{}/api/settings/country", SOCKET);
                out.write("Enter country: ");
                let country = token.next_line::<String>();
                let body = format!(r#"{{"country":"{country}"}}"#);
                (endpoint, body)
            }
            _ => {
                panic!("What the fuck bro... please enter a given option.")
            }
        };

        let res1 = client
            .post(&endpoint)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, &cookies)
            .body(body)
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
