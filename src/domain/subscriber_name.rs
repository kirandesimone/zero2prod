use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty = s.trim().is_empty();
        let is_over_char_limit = s.graphemes(true).count() > 256;

        let invalid_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let has_invalid_char = s.chars().any(|c| invalid_chars.contains(&c));

        if is_empty || is_over_char_limit || has_invalid_char {
            Err("Invalid name".into())
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

////////////////////////////////////
//// UNIT TESTS ///////////////////
//////////////////////////////////

#[cfg(test)]
mod test {
    use super::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_ok() {
        let name = "e".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_grapheme_longer_than_256_is_rejected() {
        let name = "r".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_is_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn name_containing_invalid_characters_are_rejected() {
        let invalid_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        for char in &invalid_chars {
            let name = char.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed() {
        let name = "Kiran DeSimone".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
