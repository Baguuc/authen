use serde_aux::field_attributes::deserialize_number_from_string;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String
}