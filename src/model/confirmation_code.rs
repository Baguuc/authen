use serde::{Deserialize, de::Error};

use crate::consts::{CONFIRMATION_CODE_CHAR_POOL, CONFIRMATION_CODE_LENGTH};

/// A model representing recorded confirmation code.
/// Used for querying the database.
#[derive(sqlx::FromRow, Debug)]
pub struct ConfirmationCode(String);

impl AsRef<str> for ConfirmationCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ConfirmationCode {
    pub fn parse(s: String) -> Result<Self, String> {
        if s.len() != CONFIRMATION_CODE_LENGTH {
            return Err(String::from("Registration code has invalid length."));
        }
        
        if s.chars().any(|c| !CONFIRMATION_CODE_CHAR_POOL.contains(&c)) {
            return Err(String::from("Registration code has a char from outside the char pool."));
        }

        Ok(Self(s))
    }
}

// implemented serde::Deserialize to validate the code directly in the extractor
impl<'de> Deserialize<'de> for ConfirmationCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw).map_err(D::Error::custom)
    }
}