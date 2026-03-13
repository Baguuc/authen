use authen::{model::confirmation_code::ConfirmationCode, utils::generation::generate_confirmation_token};

#[derive(Debug, Clone)]
struct ValidConfirmationCodeFixture(pub String);

impl quickcheck::Arbitrary for ValidConfirmationCodeFixture {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let code = generate_confirmation_token().unwrap();

        Self(code)
    }
}

#[quickcheck_macros::quickcheck]
fn valid_emails_are_parsed_successfully(confirmation_code: ValidConfirmationCodeFixture) -> bool {
    ConfirmationCode::parse(confirmation_code.0).is_ok()
}

#[test]
fn invalid_codes_are_rejected() {
    let invalid_codes = vec![
        "",
        "123",
        "1234567",
        "!",
        "@",
        "#"
    ];

    for code in invalid_codes {
        assert!(
            ConfirmationCode::parse(code.to_string()).is_err(),
            "{code} was parsed as a valid registration code"
        );
    }
}