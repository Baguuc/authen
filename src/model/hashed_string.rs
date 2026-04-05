/// A model representing a arbitrary hashed string.
/// Used to prevent errors with commands that expect already hashed string.
#[derive(Debug)]
pub struct HashedString(pub String);

impl AsRef<str> for HashedString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}