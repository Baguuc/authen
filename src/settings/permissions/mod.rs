use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct PermissionSettings(pub Vec<String>);

impl AsRef<Vec<String>> for PermissionSettings {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}