use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub _id: ObjectId,
    pub name: String,
    pub email: String,
    pub gender: String,
    pub dob: DateTime,
    pub username: String,
    pub password: String,
    // status: UserStatus,
    // followers: Vec<String>,
    // following: Vec<String>,
    // pub sessions: Vec<UserSession>,
    pub created: DateTime,
    pub last_login: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserStatus {
    Public,
    Private,
    Blocked,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserSession {
    token: String,
    description: String,
}

// a valid name contains two or more words
// each words should only contain english alphabets
pub fn is_name_valid(s: &str) -> Result<String, String> {
    let mut result = "".to_string();
    let mut count = 0;
    for part in s.split_whitespace() {
        if part.bytes().any(|b| !b.is_ascii_alphabetic()) {
            return Err("Only alphabets are allowed inside name".to_string());
        }
        if !result.is_empty() {
            result.push(' ');
        }
        count += 1;
        result.push_str(part);
    }
    if !result.is_empty() && count >= 2 {
        Ok(result)
    } else {
        Err("Name must contain two or more words".to_string())
    }
}

pub fn is_email_valid(s: &str) -> bool {
    let mut it = s.split('@');
    // validating email prefix
    if let Some(id) = it.next() {
        if id.len() < 6 || id.len() > 64 {
            return false;
        }
        // email prefix should only contain alphabets, digits and periods
        if !id.chars().all(|c| {
            c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_'
        }) {
            return false;
        }
        // username should start with alphabetic character
        if !id.chars().next().unwrap().is_ascii_alphabetic() {
            return false;
        }
        // username should not contain more than one period, hypen or underscore together
        if ["..", ".-", "-.", "--", "-_", "_-", "__", "._", "_."]
            .into_iter()
            .any(|p| id.contains(p))
        {
            return false;
        }
        // no period, hypen or underscore at very end
        if ['.', '-', '_']
            .iter()
            .any(|&p| p == id.chars().next_back().unwrap())
        {
            return false;
        }
    }
    // validating domain of email
    if let Some(domain) = it.next() {
        if domain.len() < 4 || domain.len() > 63 {
            return false;
        }
        let mut it = domain.split('.');
        // top level domain check
        if let Some(tld) = it.next_back() {
            if tld.len() < 2 || tld.len() > 6 || !tld.chars().all(|c| c.is_ascii_lowercase()) {
                return false;
            }
        }
        // second and third level domain check
        for _ in 0..2 {
            if let Some(ld) = it.next_back() {
                if ld.len() < 1
                    || !ld.chars().all(|c| c.is_ascii_lowercase() || c == '-')
                    || !ld.chars().next().unwrap().is_ascii_lowercase()
                    || ld.chars().next_back().unwrap() == '-'
                    || ld.contains("--")
                {
                    return false;
                }
            }
        }
        if let Some(_) = it.next_back() {
            return false;
        }
    }
    if let Some(_) = it.next() {
        return false;
    }
    true
}

pub fn into_gender(value: &mut String) {
    let _ = value
        .to_lowercase()
        .chars()
        .next()
        .unwrap()
        .to_ascii_uppercase();
    if value != "Male" && value != "Female" {
        let _ = std::mem::replace(value, String::from("Other"));
    }
}

pub fn is_username_valid(s: &str) -> Result<(), String> {
    if s.len() < 3 && s.len() > 16 {
        return Err("Username should be between 3 and 16 characters".to_string());
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        return Err("Only alphabets, digits, underscores and hyphens are allowed".to_string());
    }
    if !s.chars().next().unwrap().is_ascii_alphabetic() {
        return Err("Username should start with alphabetic character".to_string());
    }
    if ["__", "_-", "-_", "--"].into_iter().any(|p| s.contains(p)) {
        return Err(
            "Username can not contain more than one hypen or underscore together".to_string(),
        );
    }
    if s.chars().next_back().unwrap() == '-' {
        return Err("Username can't be ended with hyphen".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! name_test {
        ($($name:ident: $exp:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (haystack, expected) = $exp;
                    assert_eq!(is_name_valid(haystack).ok(), expected);
                }
            )*
        };
    }

    name_test! {
        name_test1: ("hello ", None),
        name_test2: ("   hello ", None),
        name_test3: ("   AB", None),
        name_test4: ("A", None),
        name_test5: ("SUMIT BRUH", Some("SUMIT BRUH".to_string())),
        name_test6: ("Sumit |", None),
        name_test7: (" RUST LANG", Some("RUST LANG".to_string())),
        name_test8: ("  hello  world broo  ", Some("hello world broo".to_string())),
        name_test9: ("RUST LANG   ", Some("RUST LANG".to_string())),
        name_test10: (" abc 82  cd  ", None),
        name_test11: (" abc_def  ", None),
        name_test12: (" abc-def  ", None),
        name_test13: (" abc@def  ", None),
    }

    macro_rules! email_test {
        ($($name:ident: $exp:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (haystack, expected) = $exp;
                    assert_eq!(is_email_valid(haystack), expected);
                }
            )*
        };
    }

    email_test! {
        email_test1: ("helo123@hello.com", true),
        email_test2: ("helo123@mail.google.com", true),
        email_test3: ("helo1@gmail.com", false),
        email_test4: ("helo-.123@gmail.com", false),
        email_test5: ("hello123@gmail1.com", false),
        email_test6: ("hello123@x.co7", false),
        email_test7: ("a0-0-0-0@y.x.in", true),
        email_test8: ("a0-0-0.@hello.in", false),
        email_test9: (".0.0.0@hello.in", false),
        email_test10: ("u.0..0@hello.in", false),
        email_test11: ("a1-4-7@hello.i", false),
    }

    macro_rules! username_test {
        ($($name:ident: $exp:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (haystack, expected) = $exp;
                    assert_eq!(is_username_valid(haystack).ok(), expected);
                }
            )*
        };
    }

    username_test! {
        username_test1: ("su-xe_ij_", Some(())),
        username_test2: ("su-x-_ij_", None),
        username_test3: ("su-x32-ij_", Some(())),
        username_test4: ("su-x32-", None),
        username_test5: ("ab-_re", None),
        username_test6: ("ab___resno", None),
        username_test7: ("ab---re", None),
        username_test8: ("a-7-8-8", Some(())),
    }
}
