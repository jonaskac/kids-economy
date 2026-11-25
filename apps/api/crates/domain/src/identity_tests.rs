use super::*;

#[test]
fn email_is_trimmed_and_lower_cased() {
    let email = Email::new("  Jonas@Example.COM ").expect("valid");
    assert_eq!(email.as_str(), "jonas@example.com");
}

#[test]
fn email_needs_a_local_part_an_at_and_a_domain_with_a_dot() {
    for bad in [
        "",
        "jonas",
        "@example.com",
        "jonas@",
        "jonas@example",
        "a@b@c.se",
        "jo nas@x.se",
    ] {
        assert_eq!(Email::new(bad), Err(EmailError::Invalid), "{bad:?}");
    }
    assert!(Email::new("a.b+kids@mail.example.se").is_ok());
}

#[test]
fn email_is_at_most_254_characters() {
    let long = format!("{}@example.com", "a".repeat(250));
    assert_eq!(Email::new(long), Err(EmailError::Invalid));
}

#[test]
fn password_is_10_to_128_characters_and_kept_as_typed() {
    assert_eq!(
        Password::new("short"),
        Err(PasswordError::TooShort {
            min: Password::MIN_CHARS
        })
    );
    assert_eq!(
        Password::new("x".repeat(Password::MAX_CHARS + 1)),
        Err(PasswordError::TooLong {
            max: Password::MAX_CHARS
        })
    );
    let password = Password::new(" correct horse ").expect("valid");
    assert_eq!(password.expose(), " correct horse ");
}

#[test]
fn password_never_shows_in_debug_output() {
    let password = Password::new("correct horse battery").expect("valid");
    assert!(!format!("{password:?}").contains("horse"));
}
