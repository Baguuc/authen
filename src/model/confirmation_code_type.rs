use std::fmt::Debug;

/// An enum modelling every type of confirmation code, made for better reusability.
pub enum ConfirmationCodeType {
    Registration,
    Login
}

impl AsRef<str> for ConfirmationCodeType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Registration => "registration",
            Self::Login => "login"
        }
    }
}

impl Debug for ConfirmationCodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}