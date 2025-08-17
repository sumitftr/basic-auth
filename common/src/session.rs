use cookie::{Cookie, CookieJar};

pub fn new() -> CookieJar {
    let mut jar = CookieJar::new();
    let value = uuid::Uuid::new_v4().to_string();
    let cookie = Cookie::build(("SSID", value))
        .path("/")
        .max_age(cookie::time::Duration::days(90))
        .same_site(cookie::SameSite::Strict)
        .secure(true)
        .build();
    jar.add(cookie);
    jar
}

pub fn new_as_vec() -> Vec<String> {
    let jar = new();
    jar.delta().into_iter().map(|c| c.to_string()).collect()
}
