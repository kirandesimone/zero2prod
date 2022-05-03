use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is an invalid email", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/////////////////////////////
//// UNIT TESTS/////////////
///////////////////////////

#[cfg(test)]
mod test {
    use super::SubscriberEmail;
    use claim::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::{Arbitrary, Gen};

    #[test]
    fn empty_string_is_invalid() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn missing_at_symbol_is_invalid() {
        let email = "testingdomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn missing_subject_is_invalid() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn valid_emails_parsed_are_ok() {
        let email = SafeEmail().fake();
        claim::assert_ok!(SubscriberEmail::parse(email));
    }

    // Using quickcheck for property based testing using rng input
    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl Arbitrary for ValidEmailFixture {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
