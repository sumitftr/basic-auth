use crate::AppError;

// a valid name contains two or more words
// each words should only contain english alphabets
pub fn is_name_valid(s: &str) -> Result<String, AppError> {
    let mut result = "".to_string();
    let mut count = 0;
    for part in s.split_whitespace() {
        if part.bytes().any(|b| !b.is_ascii_alphabetic()) {
            return Err(AppError::BadReq("Only alphabets are allowed inside name"));
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
        Err(AppError::BadReq("Name must contain two or more words"))
    }
}

pub fn is_email_valid(s: &str) -> Result<(), AppError> {
    let mut it = s.split('@');
    let Some(local_part) = it.next() else {
        return Err(AppError::InvalidEmailFormat);
    };
    let Some(domain) = it.next() else {
        return Err(AppError::InvalidEmailFormat);
    };
    if it.next().is_some() {
        return Err(AppError::InvalidEmailFormat);
    }

    Ok(())
}

pub fn is_username_valid(s: &str) -> Result<(), AppError> {
    if s.len() < 3 && s.len() > 16 {
        return Err(AppError::BadReq(
            "Username should be between 3 and 16 characters",
        ));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '.')
    {
        return Err(AppError::BadReq(
            "Only alphabets, digits and periods are allowed",
        ));
    }
    if !s.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(AppError::BadReq(
            "Username should start with alphabetic character",
        ));
    }
    if s.contains("..") {
        return Err(AppError::BadReq(
            "Username can't contain more than one period together",
        ));
    }
    if s.ends_with('.') {
        return Err(AppError::BadReq("Username can't be ended with a period"));
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
                    let (haystack, expected): (&str, Option<()>) = $exp;
                    assert_eq!(is_email_valid(haystack).ok(), expected);
                }
            )*
        };
    }

    email_test! {
        email_test1: ("helo123@hello.com", Some(())),
        email_test2: ("helo123@mail.google.com", Some(())),
        email_test3: ("helo1@gmail.com", None),
        email_test4: ("helo-.123@gmail.com", None),
        email_test5: ("hello123@gmail1.com", None),
        email_test6: ("hello123@x.co7", None),
        email_test7: ("a0-0-0-0@y.x.in", Some(())),
        email_test8: ("a0-0-0.@hello.in", None),
        email_test9: (".0.0.0@hello.in", None),
        email_test10: ("u.0..0@hello.in", None),
        email_test11: ("a1-4-7@hello.i", None),
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
        username_test1: ("su-xe_ij_", None),
        username_test2: ("su-x-_ij_", None),
        username_test3: ("su-x32-ij_", None),
        username_test4: ("su-x32-", None),
        username_test5: ("ab-_re", None),
        username_test6: ("ab...resno", None),
        username_test7: ("ab---re", None),
        username_test8: ("a-7-8-8", None),
        username_test9: ("a.7.b.xetn", Some(())),
    }
}
