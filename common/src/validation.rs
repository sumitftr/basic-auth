use crate::AppError;

// a valid name contains two or more words
// each words should only contain english alphabets
pub fn is_name_valid(s: &str) -> Result<String, AppError> {
    let mut result = String::new();
    let mut count = 0;
    for part in s.split_whitespace() {
        if part.chars().any(|b| !b.is_alphabetic()) {
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

    if is_local_part_valid(local_part) && is_domain_valid(domain) {
        Ok(())
    } else {
        Err(AppError::InvalidEmailFormat)
    }
}

fn is_local_part_valid(local_part: &str) -> bool {
    if local_part.is_empty() || local_part.len() > 64 {
        return false;
    }
    // local part should start with alphanumeric character
    if !local_part.chars().next().unwrap().is_ascii_alphanumeric() {
        return false;
    }
    // local part should only contain alphabets, digits and periods
    if !local_part
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        return false;
    }
    // local part should not contain more than one period, hypen or underscore together
    let allowed_symbols = ['.', '-', '_'];
    let mut it = local_part.chars();
    let mut prev = it.next().unwrap();
    for _ in 1..local_part.len() {
        let curr = it.next().unwrap();
        if allowed_symbols.contains(&prev) && allowed_symbols.contains(&curr) {
            return false;
        }
        prev = curr;
    }
    // no period, hypen or underscore at very end of local part
    if allowed_symbols
        .iter()
        .any(|&p| p == local_part.chars().next_back().unwrap())
    {
        return false;
    }
    true
}

fn is_domain_valid(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 255 || domain.contains("..") || !domain.contains(".") {
        return false;
    }
    let mut it = domain.split('.');
    // top level domain check
    if let Some(tld) = it.next_back()
        && (tld.is_empty() || !tld.chars().all(|c| c.is_ascii_alphabetic()))
    {
        return false;
    }
    // second, third and fourth level domain check
    for _ in 0..3 {
        if let Some(ld) = it.next_back()
            && (ld.is_empty()
                || !ld.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                || !ld.chars().next().unwrap().is_ascii_alphabetic()
                || ld.ends_with('-'))
        {
            return false;
        }
    }
    // no fifth level domain allowed
    if it.next_back().is_some() {
        return false;
    }
    true
}

pub fn is_username_valid(s: &str) -> Result<(), AppError> {
    if s.len() < 3 && s.len() > 16 {
        return Err(AppError::BadReq(
            "Username should be between 3 and 16 characters",
        ));
    }
    if !s.chars().next().unwrap().is_ascii_lowercase() {
        return Err(AppError::BadReq(
            "Username should start with a lowercase alphabetic character",
        ));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.')
    {
        return Err(AppError::BadReq(
            "Only lowercase alphabets, digits and periods are allowed",
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
        email_test_01: ("gggggg@example.com", Some(())),
        email_test_02: ("helo123@mail.google.com", Some(())),
        email_test_03: ("my-email@hotmail.com", Some(())),
        email_test_04: ("helo-.123@gmail.com", None),
        email_test_05: ("hello123@gma1.haha", Some(())),
        email_test_06: ("hello123@x.co7", None),
        email_test_07: ("baj-1-3@y.x.in", Some(())),
        email_test_08: ("a0-0-0.@hello.xyz", None),
        email_test_09: (".0.0.0@hello.in", None),
        email_test_10: ("u.0..0@example.in", None),
        email_test_11: ("a1-4-7@hello.i", Some(())),
        email_test_12: ("nana-7@hello", None),
        email_test_13: ("a1-@foo.rs", None),
        email_test_14: ("woosh@.foo", None),
        email_test_15: ("rosent0--7-0@y.x.in", None),
        email_test_16: ("hello0@example--com", None),
        email_test_17: ("hi@sample-.com", None),
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
        username_test_01: ("su-xe_ij_", None),
        username_test_02: ("su-x-_ij_", None),
        username_test_03: ("su-x32-ij_", None),
        username_test_04: ("su-x32-", None),
        username_test_05: ("ab-_re", None),
        username_test_06: ("ab...resno", None),
        username_test_07: ("ab---re", None),
        username_test_08: ("a-7-8-8", None),
        username_test_09: ("a.7.b.xetn", Some(())),
        username_test_10: ("example.com", Some(())),
    }
}
