use super::*;

#[test]
fn account_name_is_trimmed() {
    let name = AccountName::new("  Fun stuff ").expect("valid");
    assert_eq!(name.as_str(), "Fun stuff");
}

#[test]
fn account_name_cannot_be_blank() {
    assert_eq!(AccountName::new("   "), Err(TextError::Empty));
}

#[test]
fn account_name_has_a_maximum_length_in_characters() {
    assert!(AccountName::new("å".repeat(AccountName::MAX_CHARS)).is_ok());
    assert_eq!(
        AccountName::new("å".repeat(AccountName::MAX_CHARS + 1)),
        Err(TextError::TooLong {
            max: AccountName::MAX_CHARS
        })
    );
}

#[test]
fn memo_is_short_kid_readable_text() {
    assert_eq!(
        Memo::new(" Pocket money week 48 ").map(|m| m.as_str().to_owned()),
        Ok("Pocket money week 48".to_owned())
    );
    assert_eq!(Memo::new(""), Err(TextError::Empty));
    assert_eq!(
        Memo::new("x".repeat(Memo::MAX_CHARS + 1)),
        Err(TextError::TooLong {
            max: Memo::MAX_CHARS
        })
    );
}
