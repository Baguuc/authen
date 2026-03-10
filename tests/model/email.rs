use authen::model::email::Email;
use fake::Fake;
use fake::faker::internet::en::SafeEmail;
use fake::rand::SeedableRng;
use fake::rand::rngs::StdRng;

#[derive(Debug, Clone)]
struct ValidEmailFixture(pub String);

impl quickcheck::Arbitrary for ValidEmailFixture {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
        let email = SafeEmail().fake_with_rng(&mut rng);

        Self(email)
    }
}

#[quickcheck_macros::quickcheck]
fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
    Email::parse(valid_email.0).is_ok()
}

#[test]
fn invalid_emails_are_rejected() {
    let invalid_emails = vec![
        "",
        "plainaddress",
        "@missingusername.com",
        "username@.com",
        "username@domain..com",
    ];

    for email in invalid_emails {
        assert!(
            Email::parse(email.to_string()).is_err(),
            "{email} was parsed as a valid email"
        );
    }
}