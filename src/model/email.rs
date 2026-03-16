use serde::{Deserialize, de::Error};
use validator::ValidateEmail;

/// The model representing a valid user email.
/// Used for parsing emails.
#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Self, String> {
        if ValidateEmail::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email", s))
        }
    }
}

impl AsRef<String> for Email {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

// implemented serde::Deserialize to validate the email directly in the extractor
impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw).map_err(D::Error::custom)    
    }
}