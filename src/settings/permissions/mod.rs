use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct PermissionSettings(pub Vec<String>);