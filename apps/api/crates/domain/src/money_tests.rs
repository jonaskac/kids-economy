use super::*;

fn sek() -> Currency {
    Currency::new("SEK").expect("valid currency")
}

#[test]
fn currency_is_a_three_letter_iso_code() {
    assert_eq!(sek().code(), "SEK");
    assert_eq!(
        Currency::new("eur").map(|c| c.code().to_owned()),
        Ok("EUR".to_owned())
    );
    assert_eq!(Currency::new("SE"), Err(MoneyError::InvalidCurrency));
    assert_eq!(Currency::new("SEKK"), Err(MoneyError::InvalidCurrency));
    assert_eq!(Currency::new("S3K"), Err(MoneyError::InvalidCurrency));
}

#[test]
fn amount_is_positive_integer_minor_units() {
    let amount = Amount::new(1250, sek()).expect("valid amount");
    assert_eq!(amount.minor(), 1250);
    assert_eq!(amount.currency(), sek());
}

#[test]
fn a_zero_amount_is_never_recorded() {
    assert_eq!(Amount::new(0, sek()), Err(MoneyError::NotPositive));
}

#[test]
fn an_amount_cannot_be_negative() {
    assert_eq!(Amount::new(-1, sek()), Err(MoneyError::NotPositive));
}

#[test]
fn balance_starts_at_zero_in_its_currency() {
    let balance = Balance::zero(sek());
    assert_eq!(balance.minor(), 0);
    assert_eq!(balance.currency(), sek());
}

#[test]
fn balance_covers_an_amount_only_when_it_is_at_least_as_large() {
    let balance = Balance::zero(sek()).plus(Amount::new(500, sek()).expect("amount"));
    assert_eq!(balance.map(|b| b.minor()), Some(500));
    let balance = balance.expect("balance");
    assert!(balance.covers(Amount::new(500, sek()).expect("amount")));
    assert!(!balance.covers(Amount::new(501, sek()).expect("amount")));
}

#[test]
fn balance_arithmetic_reports_overflow_instead_of_wrapping() {
    let max = Amount::new(i64::MAX, sek()).expect("amount");
    let full = Balance::zero(sek()).plus(max).expect("fits");
    assert_eq!(full.plus(Amount::new(1, sek()).expect("amount")), None);
}
